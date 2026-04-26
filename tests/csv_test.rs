// =============================================================================
// tests/csv_test.rs
// Auteure : NZEUTEM DOMMOE Eunice Felixtine (#02)
//
// Tests d'INTÉGRATION pour le CsvReader.
// Contrairement aux tests unitaires (dans csv_reader.rs), ces tests vérifient
// le comportement du module VU DE L'EXTÉRIEUR — exactement comme le ferait
// l'orchestrateur (pipeline.rs) ou un autre membre du groupe.
//
// Pour lancer uniquement ces tests :
//   cargo test --test csv_test
// =============================================================================

// On importe le module reader de datapipe comme le ferait n'importe quel
// autre fichier du projet. C'est ça la différence clé avec les tests unitaires.
use datapipe::reader::{SourceReader, Record};
use datapipe::reader::csv_reader::CsvReader;
use serde_json::{json, Value};
use std::io::Write;
use tempfile::NamedTempFile;

// --- Fonction helper partagée par tous les tests ----------------------------
fn creer_csv_temp(contenu: &str) -> NamedTempFile {
    let mut f = NamedTempFile::new().expect("Impossible de créer fichier temporaire");
    write!(f, "{}", contenu).expect("Impossible d'écrire dans le fichier temporaire");
    f
}

// =============================================================================
// GROUPE 1 — Vérification du contrat SourceReader
// Ces tests vérifient que CsvReader respecte bien le trait.
// =============================================================================

/// Vérifie que CsvReader peut être utilisé à travers le trait SourceReader.
/// C'est ce que fera l'orchestrateur : il ne connaît que le trait, pas le type concret.
#[test]
fn test_csvreader_respecte_trait_sourcereader() {
    let contenu = "nom,age\nJean,25";
    let fichier = creer_csv_temp(contenu);

    // On stocke dans une variable de type &dyn SourceReader
    // (le trait, pas le type concret) — c'est exactement ce que fait le pipeline
    let lecteur: &dyn SourceReader = &CsvReader::new(fichier.path().to_str().unwrap());
    let records: Vec<_> = lecteur.records().collect();

    assert_eq!(records.len(), 1);
    assert!(records[0].is_ok());
}

/// Vérifie qu'on peut itérer plusieurs fois sur des instances différentes.
#[test]
fn test_plusieurs_instances_independantes() {
    let contenu = "id,valeur\n1,A\n2,B";
    let fichier = creer_csv_temp(contenu);
    let chemin = fichier.path().to_str().unwrap();

    let lecteur1 = CsvReader::new(chemin);
    let lecteur2 = CsvReader::new(chemin);

    let records1: Vec<_> = lecteur1.records().collect();
    let records2: Vec<_> = lecteur2.records().collect();

    // Les deux doivent retourner les mêmes données
    assert_eq!(records1.len(), records2.len());
    assert_eq!(records1.len(), 2);
}

// =============================================================================
// GROUPE 2 — Tests avec les vraies données camerounaises du projet
// =============================================================================

/// Test de bout en bout avec le fichier etudiants_ensp.csv réel.
/// Ce test vérifie que le fichier de données livré avec le projet est bien lisible.
#[test]
fn test_fichier_etudiants_ensp_reel() {
    // Ce test utilise le vrai fichier de données du projet
    let lecteur = CsvReader::new("data/etudiants_ensp.csv");
    let records: Vec<_> = lecteur.records().collect();

    // Le fichier doit avoir exactement 10 lignes (10 membres du groupe)
    assert_eq!(records.len(), 10, "Le fichier etudiants_ensp.csv doit avoir 10 étudiants");

    // Tous les records doivent être Ok (aucune erreur de lecture)
    for (i, record) in records.iter().enumerate() {
        assert!(record.is_ok(), "La ligne {} doit être lisible sans erreur", i + 1);
    }

    // Vérifier que notre auteure est bien dans les données
    let nzeutem = records.iter()
        .find(|r| {
            if let Ok(rec) = r {
                rec.get("matricule") == Some(&Value::String("22G00347".to_string()))
            } else {
                false
            }
        });
    assert!(nzeutem.is_some(), "NZEUTEM (22G00347) doit être dans le fichier");
}

/// Test avec le fichier patients_hopital.csv — vérifie le cas d'usage médical.
#[test]
fn test_fichier_patients_hopital_reel() {
    let lecteur = CsvReader::new("data/patients_hopital.csv");
    let records: Vec<_> = lecteur
        .records()
        .map(|r| r.expect("Ligne invalide dans patients_hopital.csv"))
        .collect();

    assert_eq!(records.len(), 30, "Le fichier doit avoir 30 patients");

    // Vérifier les colonnes attendues sur le premier patient
    let p1 = &records[0];
    assert!(p1.contains_key("id_patient"), "Colonne id_patient manquante");
    assert!(p1.contains_key("NomPatient"), "Colonne NomPatient manquante");
    assert!(p1.contains_key("departement"), "Colonne departement manquante");
    assert!(p1.contains_key("cout_fcfa"), "Colonne cout_fcfa manquante");

    // Le coût doit être un nombre (détection automatique)
    assert!(
        matches!(p1["cout_fcfa"], Value::Number(_)),
        "cout_fcfa doit être détecté comme nombre, pas comme texte"
    );

    // Le champ assure doit être un booléen
    assert!(
        matches!(p1["assure"], Value::Bool(_)),
        "assure doit être détecté comme booléen"
    );
}

/// Test avec le fichier ventes_marche.csv — délimiteur point-virgule.
#[test]
fn test_fichier_ventes_marche_point_virgule() {
    let lecteur = CsvReader::with_delimiter("data/ventes_marche.csv", ';');
    let records: Vec<_> = lecteur
        .records()
        .map(|r| r.expect("Ligne invalide dans ventes_marche.csv"))
        .collect();

    assert_eq!(records.len(), 30, "Le fichier doit avoir 30 ventes");

    // Vérifier que les colonnes sont bien séparées par le ';'
    // Si le délimiteur était ',' par erreur, tout serait dans une seule colonne
    assert!(records[0].contains_key("vendeur"), "Colonne vendeur manquante");
    assert!(records[0].contains_key("produit"), "Colonne produit manquante");
    assert!(records[0].contains_key("prix_unitaire_fcfa"), "Colonne prix_unitaire_fcfa manquante");

    // Le prix doit être un nombre
    assert!(
        matches!(records[0]["prix_unitaire_fcfa"], Value::Number(_)),
        "prix_unitaire_fcfa doit être un nombre"
    );
}

// =============================================================================
// GROUPE 3 — Simulation du pipeline ETL complet
// Ces tests simulent ce que TSEFACK (pipeline.rs) fera avec notre lecteur.
// =============================================================================

/// Simule le pipeline : lire CSV → filtrer → compter.
/// Reproduit l'usage que fera l'orchestrateur de notre lecteur.
#[test]
fn test_simulation_pipeline_filtrage() {
    // Données : résultats BEPC avec mentions différentes
    let contenu = "nom,note,mention\n\
        Alpha,18.5,Très bien\n\
        Béta,14.2,Bien\n\
        Gamma,12.0,Assez bien\n\
        Delta,9.8,Passable\n\
        Epsilon,7.5,Insuffisant\n\
        Zeta,15.0,Bien";

    let fichier = creer_csv_temp(contenu);
    let lecteur = CsvReader::new(fichier.path().to_str().unwrap());

    // Simuler un filtre : garder seulement les mentions "Bien"
    let bien: Vec<Record> = lecteur
        .records()
        .filter_map(|r| r.ok())
        .filter(|rec| {
            rec.get("mention") == Some(&Value::String("Bien".to_string()))
        })
        .collect();

    assert_eq!(bien.len(), 2, "Seulement Alpha et Zeta ont la mention Bien... non : Béta et Zeta");
    // Vérifions les noms
    let noms: Vec<&Value> = bien.iter().map(|r| &r["nom"]).collect();
    assert!(noms.contains(&&Value::String("Béta".to_string())));
    assert!(noms.contains(&&Value::String("Zeta".to_string())));
}

/// Simule le pipeline : lire CSV → calculer une somme.
/// Reproduit ce que fera ComputeTransform (NOLACK #05).
#[test]
fn test_simulation_pipeline_calcul_somme() {
    let contenu = "produit,quantite,prix_unitaire\n\
        Tomate,10,600\n\
        Poisson,3,2500\n\
        Plantain,20,300";

    let fichier = creer_csv_temp(contenu);
    let lecteur = CsvReader::new(fichier.path().to_str().unwrap());

    // Calculer le total des ventes (quantite * prix_unitaire)
    let total: f64 = lecteur
        .records()
        .filter_map(|r| r.ok())
        .map(|rec| {
            let qte = match &rec["quantite"] {
                Value::Number(n) => n.as_f64().unwrap_or(0.0),
                _ => 0.0,
            };
            let prix = match &rec["prix_unitaire"] {
                Value::Number(n) => n.as_f64().unwrap_or(0.0),
                _ => 0.0,
            };
            qte * prix
        })
        .sum();

    // 10*600 + 3*2500 + 20*300 = 6000 + 7500 + 6000 = 19500
    assert_eq!(total, 19500.0, "Le total des ventes doit être 19500 FCFA");
}

/// Simule le pipeline : lire CSV → renommer colonnes.
/// Reproduit ce que fera RenameTransform (ASSONGUE #04).
#[test]
fn test_simulation_pipeline_renommage() {
    // Fichier avec noms de colonnes "techniques" à nettoyer
    let contenu = "NomPatient,DateNaiss,departement\n\
        ATEBA Jean,1985-03-12,Médecine interne\n\
        MVOGO Marie,1992-07-25,Pédiatrie";

    let fichier = creer_csv_temp(contenu);
    let lecteur = CsvReader::new(fichier.path().to_str().unwrap());

    // Simuler un renommage : NomPatient → nom, DateNaiss → date_naissance
    let records: Vec<Record> = lecteur
        .records()
        .filter_map(|r| r.ok())
        .map(|mut rec| {
            // Renommage manuel (ce que RenameTransform fera automatiquement)
            if let Some(val) = rec.shift_remove("NomPatient") {
                rec.insert("nom".to_string(), val);
            }
            if let Some(val) = rec.shift_remove("DateNaiss") {
                rec.insert("date_naissance".to_string(), val);
            }
            rec
        })
        .collect();

    assert_eq!(records.len(), 2);
    // Les anciennes colonnes ne doivent plus exister
    assert!(!records[0].contains_key("NomPatient"), "NomPatient doit avoir été renommé");
    assert!(!records[0].contains_key("DateNaiss"), "DateNaiss doit avoir été renommé");
    // Les nouvelles colonnes doivent exister
    assert!(records[0].contains_key("nom"), "La colonne 'nom' doit exister");
    assert!(records[0].contains_key("date_naissance"), "La colonne 'date_naissance' doit exister");
}

// =============================================================================
// GROUPE 4 — Tests des cas limites (edge cases)
// =============================================================================

/// Un CSV avec une seule colonne.
#[test]
fn test_csv_une_seule_colonne() {
    let contenu = "region\nCentre\nLittoral\nOuest\nNord";

    let fichier = creer_csv_temp(contenu);
    let lecteur = CsvReader::new(fichier.path().to_str().unwrap());
    let records: Vec<_> = lecteur.records().map(|r| r.unwrap()).collect();

    assert_eq!(records.len(), 4);
    assert_eq!(records[0]["region"], Value::String("Centre".to_string()));
    assert_eq!(records[3]["region"], Value::String("Nord".to_string()));
}

/// Un CSV avec beaucoup de colonnes — vérifie la robustesse.
#[test]
fn test_csv_nombreuses_colonnes() {
    let en_tetes = "c1,c2,c3,c4,c5,c6,c7,c8,c9,c10";
    let valeurs  = "1,2,3,4,5,6,7,8,9,10";
    let contenu  = format!("{}\n{}", en_tetes, valeurs);

    let fichier = creer_csv_temp(&contenu);
    let lecteur = CsvReader::new(fichier.path().to_str().unwrap());
    let records: Vec<_> = lecteur.records().map(|r| r.unwrap()).collect();

    assert_eq!(records.len(), 1);
    assert_eq!(records[0].len(), 10, "Le record doit avoir 10 colonnes");
    assert_eq!(records[0]["c1"], json!(1));
    assert_eq!(records[0]["c10"], json!(10));
}

/// Vérifie que le mauvais délimiteur ne "casse" pas tout,
/// mais retourne une seule colonne par ligne.
#[test]
fn test_mauvais_delimiteur_retourne_une_colonne() {
    // Fichier séparé par ';' mais on utilise ',' comme délimiteur
    let contenu = "nom;age;ville\nJean;25;Douala";

    let fichier = creer_csv_temp(contenu);
    // On utilise ',' alors que le vrai séparateur est ';'
    let lecteur = CsvReader::new(fichier.path().to_str().unwrap()); // virgule par défaut
    let records: Vec<_> = lecteur.records().map(|r| r.unwrap()).collect();

    // Avec le mauvais délimiteur, toute la ligne est dans une seule "colonne"
    // (la colonne s'appelle "nom;age;ville")
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].len(), 1, "Avec le mauvais délimiteur, tout est dans 1 colonne");
}
