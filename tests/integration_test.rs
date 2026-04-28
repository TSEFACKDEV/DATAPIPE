// tests/integration_test.rs
//
// Auteur : NGANSOP NGOUABOU FREDI LOIK (#07)
// Rôle   : Tests d'intégration complets pour le projet DataPipe
//
// ─── DIFFÉRENCE ENTRE TESTS UNITAIRES ET TESTS D'INTÉGRATION ────────────────
//
// Tests UNITAIRES (dans les fichiers src/) :
//   - Testent UN composant en isolation
//   - Ex: "CsvSinkWriter écrit-il une ligne correctement ?"
//   - Rapides à exécuter
//
// Tests d'INTÉGRATION (ce fichier) :
//   - Testent PLUSIEURS composants qui collaborent ensemble
//   - Ex: "Un pipeline CSV → transformations → JSONL fonctionne-t-il bout à bout ?"
//   - Simulent le comportement réel de l'application
//   - Détectent les bugs d'interface entre modules
//
// ─── ORGANISATION DES TESTS ──────────────────────────────────────────────────
//
// Ce fichier est placé dans tests/ (et non src/) car il s'agit de tests
// "boîte noire" : on teste l'API publique de datapipe comme le ferait
// un utilisateur externe, sans accéder aux internaux des modules.
//
// En Rust, les fichiers dans tests/ ont automatiquement accès à tous les
// items `pub` de la crate, mais pas aux items `pub(crate)` ou privés.
//
// ─── STRATÉGIE DE TEST ───────────────────────────────────────────────────────
// On utilise des fichiers temporaires (tempfile::tempdir) pour éviter :
//   1. La pollution du répertoire de travail
//   2. Les conflits entre tests parallèles (Rust exécute les tests en //)
//   3. La nécessité de nettoyage manuel
// tempdir() crée un dossier unique dans /tmp et le supprime automatiquement
// quand la variable est droppée (RAII pattern).
// ────────────────────────────────────────────────────────────────────────────

use datapipe::reader::Record;
use datapipe::writer::SinkWriter;
use datapipe::writer::factory::create_writer;
use datapipe::writer::jsonl_writer::JsonLinesSinkWriter;
use datapipe::config::DestinationConfig;
use serde_json::{json, Value};
use tempfile::tempdir;

// ─── HELPERS PARTAGÉS ────────────────────────────────────────────────────────

/// Crée un Record de test simple à partir de paires (clé, valeur JSON).
fn make_record(pairs: &[(&str, Value)]) -> Record {
    pairs.iter().map(|(k, v)| (k.to_string(), v.clone())).collect()
}

/// Crée un Record représentant un employé pour les tests métier.
fn make_employee_record(nom: &str, age: i64, dept: &str, salaire: f64) -> Record {
    make_record(&[
        ("nom", json!(nom)),
        ("age", json!(age)),
        ("departement", json!(dept)),
        ("salaire", json!(salaire)),
    ])
}

/// Lit un fichier JSONL et retourne le vecteur de records parsés.
/// Chaque ligne doit être un objet JSON valide.
fn read_jsonl_file(path: &str) -> Vec<Value> {
    let content = std::fs::read_to_string(path)
        .unwrap_or_else(|_| panic!("Impossible de lire le fichier JSONL : {}", path));
    
    content
        .lines()
        .filter(|l| !l.is_empty())
        .enumerate()
        .map(|(i, line)| {
            serde_json::from_str(line)
                .unwrap_or_else(|e| panic!("Ligne {} n'est pas du JSON valide: {} → {}", i+1, line, e))
        })
        .collect()
}

// ═══════════════════════════════════════════════════════════════════════════════
// SECTION 1 : TESTS JSONL WRITER (composant de NGANSOP)
// ═══════════════════════════════════════════════════════════════════════════════

/// Test 01 : L'écrivain JSONL écrit des records valides
#[test]
fn test_jsonl_writer_ecriture_basique() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("output.jsonl").to_str().unwrap().to_string();

    let mut writer = JsonLinesSinkWriter::new(&path).unwrap();

    let records = vec![
        make_employee_record("Jean Mbarga", 25, "Informatique", 50000.0),
        make_employee_record("Marie Nkeng", 30, "RH", 45000.0),
        make_employee_record("Paul Fotso", 35, "Informatique", 55000.0),
    ];

    for record in &records {
        writer.write_record(record).unwrap();
    }
    writer.finalize().unwrap();

    let parsed = read_jsonl_file(&path);
    assert_eq!(parsed.len(), 3, "Doit avoir 3 lignes pour 3 records");
    
    // Vérifier les noms dans l'ordre
    let noms_attendus = ["Jean Mbarga", "Marie Nkeng", "Paul Fotso"];
    for (i, record) in parsed.iter().enumerate() {
        assert_eq!(record["nom"], json!(noms_attendus[i]));
    }
}

/// Test 02 : Format JSONL strict → 1 ligne = 1 objet JSON valide
#[test]
fn test_jsonl_format_strict_une_ligne_par_record() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("strict.jsonl").to_str().unwrap().to_string();

    let mut writer = JsonLinesSinkWriter::new(&path).unwrap();

    // Record avec des valeurs complexes (imbriqué ne devrait pas arriver en ETL,
    // mais testons quand même la sérialisation)
    let record = make_record(&[
        ("nom", json!("Test avec, virgule")),
        ("description", json!("Valeur avec \"guillemets\"")),
        ("score", json!(99.99)),
    ]);
    writer.write_record(&record).unwrap();
    writer.finalize().unwrap();

    let content = std::fs::read_to_string(&path).unwrap();
    let lignes: Vec<&str> = content.lines().collect();
    
    // Exactement 1 ligne non vide
    let non_vides: Vec<&&str> = lignes.iter().filter(|l| !l.is_empty()).collect();
    assert_eq!(non_vides.len(), 1, "Un record = une ligne");
    
    // La ligne est du JSON valide
    let _: Value = serde_json::from_str(non_vides[0])
        .expect("La ligne doit être du JSON valide");
}

// ═══════════════════════════════════════════════════════════════════════════════
// SECTION 2 : TESTS FACTORY 
// ═══════════════════════════════════════════════════════════════════════════════

/// Test 03 : La factory crée le bon type d'écrivain pour "jsonl"
#[test]
fn test_factory_format_jsonl() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("factory_test.jsonl").to_str().unwrap().to_string();

    let config = DestinationConfig {
        format: "jsonl".to_string(),
        path: path.clone(),
    };

    let mut writer = create_writer(&config).expect("factory doit créer un writer JSONL");

    let record = make_record(&[("test", json!("factory_jsonl"))]);
    writer.write_record(&record).unwrap();
    writer.finalize().unwrap();

    let parsed = read_jsonl_file(&path);
    assert_eq!(parsed.len(), 1);
    assert_eq!(parsed[0]["test"], json!("factory_jsonl"));
}

/// Test 04 : La factory crée le bon type d'écrivain pour "json"
#[test]
fn test_factory_format_json() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("factory_test.json").to_str().unwrap().to_string();

    let config = DestinationConfig {
        format: "json".to_string(),
        path: path.clone(),
    };

    let mut writer = create_writer(&config).expect("factory doit créer un writer JSON");

    let record = make_record(&[("test", json!("factory_json"))]);
    writer.write_record(&record).unwrap();
    writer.finalize().unwrap();

    // Le JSON doit être un tableau valide
    let content = std::fs::read_to_string(&path).unwrap();
    let parsed: Value = serde_json::from_str(&content)
        .expect("Le fichier JSON doit être valide");
    assert!(parsed.is_array(), "Le fichier JSON doit contenir un tableau");
    assert_eq!(parsed[0]["test"], json!("factory_json"));
}

/// Test 05 : La factory crée le bon type d'écrivain pour "csv"
#[test]
fn test_factory_format_csv() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("factory_test.csv").to_str().unwrap().to_string();

    let config = DestinationConfig {
        format: "csv".to_string(),
        path: path.clone(),
    };

    let mut writer = create_writer(&config).expect("factory doit créer un writer CSV");

    let record = make_record(&[("nom", json!("Test")), ("age", json!(25))]);
    writer.write_record(&record).unwrap();
    writer.finalize().unwrap();

    let content = std::fs::read_to_string(&path).unwrap();
    assert!(!content.is_empty(), "Le fichier CSV ne doit pas être vide");
    // Au minimum, le fichier doit contenir les en-têtes
    assert!(content.contains("nom") || content.contains("age"));
}

/// Test 06 : Format inconnu → erreur claire
#[test]
fn test_factory_format_inconnu_erreur() {
    let config = DestinationConfig {
        format: "xml".to_string(),
        path: "/tmp/test.xml".to_string(),
    };

    match create_writer(&config) {
        Err(e) => {
            let msg = format!("{}", e);
            assert!(msg.contains("xml"), "L'erreur doit mentionner 'xml' : {}", msg);
        }
        Ok(_) => panic!("Un format inconnu doit retourner une erreur"),
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SECTION 3 : TESTS D'INTÉGRATION PIPELINE BOUT EN BOUT
// ═══════════════════════════════════════════════════════════════════════════════

/// Test 07 : Pipeline CSV → JSONL avec simulation de pipeline complet
///
/// Ce test simule ce que pipeline.rs fait :
/// 1. Lire des records (ici on les crée directement en mémoire)
/// 2. Appliquer des "transformations" (ici on les code manuellement)
/// 3. Écrire en JSONL
///
/// Une fois TSEFACK (#01) et NZEUTEM (#02) auront complété leur travail,
/// ce test pourra utiliser les vraies implémentations.
#[test]
fn test_pipeline_records_en_memoire_vers_jsonl() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("pipeline_output.jsonl").to_str().unwrap().to_string();

    // Simulation : records "lus" depuis une source CSV
    let records_source = vec![
        make_employee_record("Jean Mbarga", 25, "Informatique", 50000.0),
        make_employee_record("Marie Nkeng", 30, "RH", 45000.0),
        make_employee_record("Paul Fotso", 35, "Informatique", 55000.0),
        make_employee_record("Aline Biya", 28, "Finance", 48000.0),
        make_employee_record("Thomas Essola", 40, "Informatique", 65000.0),
    ];

    // Simulation : transformation FILTER (garder seulement "Informatique")
    let records_filtres: Vec<Record> = records_source
        .into_iter()
        .filter(|r| r.get("departement") == Some(&json!("Informatique")))
        .collect();

    // Simulation : transformation COMPUTE (ajouter colonne "prime" = salaire * 0.1)
    let records_transformes: Vec<Record> = records_filtres
        .into_iter()
        .map(|mut r| {
            let prime = r.get("salaire")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0) * 0.1;
            r.insert("prime".to_string(), json!(prime));
            r
        })
        .collect();

    // Simulation : transformation DROP (supprimer "salaire")
    let records_finaux: Vec<Record> = records_transformes
        .into_iter()
        .map(|mut r| { r.shift_remove("salaire"); r })
        .collect();

    // ÉCRITURE → rôle de NGANSOP
    let mut writer = JsonLinesSinkWriter::new(&path).unwrap();
    for record in &records_finaux {
        writer.write_record(record).unwrap();
    }
    writer.finalize().unwrap();

    // VÉRIFICATIONS
    let parsed = read_jsonl_file(&path);

    // Seulement les records "Informatique" (3 sur 5)
    assert_eq!(parsed.len(), 3, "Le filtre Informatique doit garder 3 records");

    // Vérifier que "salaire" a été supprimé par DROP
    for record in &parsed {
        assert!(
            record.get("salaire").is_none(),
            "La colonne 'salaire' doit avoir été supprimée par DROP"
        );
    }

    // Vérifier que "prime" a été calculé
    for record in &parsed {
        assert!(
            record.get("prime").is_some(),
            "La colonne 'prime' doit exister (ajoutée par COMPUTE)"
        );
        let prime = record["prime"].as_f64().unwrap();
        assert!(prime > 0.0, "La prime doit être positive");
    }

    // Vérifier le departement
    for record in &parsed {
        assert_eq!(record["departement"], json!("Informatique"));
    }
}

/// Test 08 : Pipeline avec 1000 records simulés (test de charge basique)
///
/// Objectif : vérifier que l'écrivain JSONL gère les gros volumes sans
/// crash mémoire ni erreur d'I/O.
#[test]
fn test_pipeline_1000_records_jsonl() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("charge_1000.jsonl").to_str().unwrap().to_string();

    let mut writer = JsonLinesSinkWriter::new(&path).unwrap();

    // Génération de 1000 records réalistes
    let departements = ["Informatique", "RH", "Finance", "Direction", "Marketing"];
    let villes = ["Douala", "Yaoundé", "Bafoussam", "Garoua", "Bamenda"];

    for i in 0..1000usize {
        let record = make_record(&[
            ("id", json!(i + 1)),
            ("nom", json!(format!("Employe_{:04}", i + 1))),
            ("age", json!(22 + (i % 40) as i64)),
            ("departement", json!(departements[i % departements.len()])),
            ("salaire", json!(30000.0 + (i as f64) * 25.5)),
            ("ville", json!(villes[i % villes.len()])),
            ("actif", json!(i % 7 != 0)), // ~85% actifs
        ]);
        writer.write_record(&record).unwrap();
    }
    writer.finalize().unwrap();

    // Vérifications
    let content = std::fs::read_to_string(&path).unwrap();
    let nb_lignes = content.lines().count();
    assert_eq!(nb_lignes, 1000, "Doit avoir exactement 1000 lignes");

    // Vérifier la validité JSON de chaque ligne
    for (i, ligne) in content.lines().enumerate() {
        let parsed: Value = serde_json::from_str(ligne)
            .unwrap_or_else(|e| panic!("Ligne {} invalide: {}", i+1, e));
        assert_eq!(parsed["id"], json!(i + 1));
    }
}

/// Test 09 : Pipeline avec filtre (vérifier que certains records sont rejetés)
///
/// Valide que le pipeline peut correctement rejeter des records et que seuls
/// les records conformes apparaissent dans la sortie JSONL.
#[test]
fn test_pipeline_avec_filtre_records_rejetes() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("filtre_test.jsonl").to_str().unwrap().to_string();

    let records_source = vec![
        make_record(&[("nom", json!("A")), ("score", json!(90))]),
        make_record(&[("nom", json!("B")), ("score", json!(45))]), // à filtrer
        make_record(&[("nom", json!("C")), ("score", json!(78))]),
        make_record(&[("nom", json!("D")), ("score", json!(30))]), // à filtrer
        make_record(&[("nom", json!("E")), ("score", json!(88))]),
    ];

    // Simulation FILTER : garder score >= 60
    let seuil = 60.0;
    let mut nb_filtres = 0;

    let mut writer = JsonLinesSinkWriter::new(&path).unwrap();

    for record in records_source {
        let score = record.get("score")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        if score >= seuil {
            writer.write_record(&record).unwrap();
        } else {
            nb_filtres += 1;
        }
    }
    writer.finalize().unwrap();

    // 2 records filtrés (B avec 45, D avec 30)
    assert_eq!(nb_filtres, 2, "2 records doivent être filtrés (score < 60)");

    // 3 records écrits
    let parsed = read_jsonl_file(&path);
    assert_eq!(parsed.len(), 3, "3 records doivent passer le filtre");

    // Tous les records écrits ont score >= 60
    for record in &parsed {
        let score = record["score"].as_f64().unwrap();
        assert!(score >= seuil, "Tous les records écrits doivent avoir score >= {}", seuil);
    }
}

/// Test 10 : Pipeline rename + compute + drop via JSONL
///
/// Simule une séquence de transformations typique en entreprise :
/// - rename : normaliser les noms de colonnes
/// - compute : calculer une colonne dérivée
/// - drop : supprimer les données sensibles
#[test]
fn test_pipeline_rename_compute_drop_vers_jsonl() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("rcd_test.jsonl").to_str().unwrap().to_string();

    // Source simulée avec colonnes "brutes"
    let records_source: Vec<Record> = vec![
        make_record(&[
            ("nom_complet", json!("Jean Mbarga")),  // sera renommé → "nom"
            ("remuneration", json!(50000.0)),         // sera renommé → "salaire"
            ("code_secret", json!("XY7Z")),           // sera supprimé (DROP)
        ]),
        make_record(&[
            ("nom_complet", json!("Marie Nkeng")),
            ("remuneration", json!(45000.0)),
            ("code_secret", json!("AB1C")),
        ]),
    ];

    let mut writer = JsonLinesSinkWriter::new(&path).unwrap();

    for record in records_source {
        // Simulation RENAME: nom_complet → nom
        let mut r: Record = indexmap::IndexMap::new();
        if let Some(v) = record.get("nom_complet") {
            r.insert("nom".to_string(), v.clone());
        }
        if let Some(v) = record.get("remuneration") {
            r.insert("salaire".to_string(), v.clone());
        }
        // Simulation COMPUTE: prime = salaire * 0.15
        let salaire = r.get("salaire").and_then(|v| v.as_f64()).unwrap_or(0.0);
        r.insert("prime".to_string(), json!(salaire * 0.15));
        // Simulation DROP: ne pas copier "code_secret"
        // (déjà absent car on ne l'a pas copié)

        writer.write_record(&r).unwrap();
    }
    writer.finalize().unwrap();

    let parsed = read_jsonl_file(&path);
    assert_eq!(parsed.len(), 2);

    for record in &parsed {
        // "nom" existe (renommé depuis nom_complet)
        assert!(record.get("nom").is_some(), "La colonne 'nom' doit exister");
        // "salaire" existe (renommé depuis remuneration)
        assert!(record.get("salaire").is_some(), "La colonne 'salaire' doit exister");
        // "prime" calculé
        assert!(record.get("prime").is_some(), "La colonne 'prime' doit exister");
        // "code_secret" supprimé
        assert!(record.get("code_secret").is_none(), "'code_secret' ne doit pas exister");
        // "nom_complet" et "remuneration" supprimés (renommés)
        assert!(record.get("nom_complet").is_none(), "'nom_complet' ne doit pas exister");
        assert!(record.get("remuneration").is_none(), "'remuneration' ne doit pas exister");
        // Vérifier que prime = salaire * 0.15
        let salaire = record["salaire"].as_f64().unwrap();
        let prime = record["prime"].as_f64().unwrap();
        let diff = (prime - salaire * 0.15).abs();
        assert!(diff < 0.01, "prime doit valoir salaire * 0.15");
    }
}

/// Test 11 : Vérification des statistiques d'exécution simulées
///
/// Simule le comptage que ExecutionStats doit réaliser pendant un pipeline.
#[test]
fn test_statistiques_pipeline_simule() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("stats_test.jsonl").to_str().unwrap().to_string();

    let nb_source = 50usize;
    let nb_filtres = 15usize; // 15 records seront rejetés
    let nb_attendu_ecrits = nb_source - nb_filtres;

    let mut records_lus = 0usize;
    let mut records_filtres_count = 0usize;
    let mut records_ecrits = 0usize;

    let mut writer = JsonLinesSinkWriter::new(&path).unwrap();

    for i in 0..nb_source {
        records_lus += 1;

        // Simuler un filtre : rejeter les records où id % 3 == 0 (jusqu'à 15 rejetés)
        if i % (nb_source / nb_filtres) == 0 && records_filtres_count < nb_filtres {
            records_filtres_count += 1;
            continue; // Filtré → pas écrit
        }

        let record = make_record(&[("id", json!(i)), ("valeur", json!(i * 100))]);
        writer.write_record(&record).unwrap();
        records_ecrits += 1;
    }
    writer.finalize().unwrap();

    // Vérifications des compteurs
    assert_eq!(records_lus, nb_source);
    assert_eq!(records_filtres_count, nb_filtres);
    assert_eq!(records_ecrits, nb_attendu_ecrits);

    // Vérifier que le fichier contient bien le bon nombre de records
    let content = std::fs::read_to_string(&path).unwrap();
    let nb_lignes = content.lines().count();
    assert_eq!(nb_lignes, records_ecrits,
        "Le nombre de lignes JSONL doit correspondre aux records écrits");
}

/// Test 12 : Cohérence CSV → JSONL (même données, deux formats)
///
/// Teste que les mêmes records produits en JSONL et en JSON contiennent
/// les mêmes données, garantissant la cohérence de la factory.
#[test]
fn test_coherence_json_vs_jsonl() {
    let dir = tempdir().unwrap();
    let jsonl_path = dir.path().join("coherence.jsonl").to_str().unwrap().to_string();
    let json_path = dir.path().join("coherence.json").to_str().unwrap().to_string();

    let records = vec![
        make_record(&[("id", json!(1)), ("nom", json!("Alpha")), ("score", json!(95.5))]),
        make_record(&[("id", json!(2)), ("nom", json!("Beta")),  ("score", json!(87.2))]),
        make_record(&[("id", json!(3)), ("nom", json!("Gamma")), ("score", json!(72.8))]),
    ];

    // Écriture JSONL
    let config_jsonl = DestinationConfig {
        format: "jsonl".to_string(),
        path: jsonl_path.clone(),
    };
    let mut writer_jsonl = create_writer(&config_jsonl).unwrap();
    for r in &records { writer_jsonl.write_record(r).unwrap(); }
    writer_jsonl.finalize().unwrap();

    // Écriture JSON
    let config_json = DestinationConfig {
        format: "json".to_string(),
        path: json_path.clone(),
    };
    let mut writer_json = create_writer(&config_json).unwrap();
    for r in &records { writer_json.write_record(r).unwrap(); }
    writer_json.finalize().unwrap();

    // Lecture JSONL
    let parsed_jsonl = read_jsonl_file(&jsonl_path);

    // Lecture JSON
    let json_content = std::fs::read_to_string(&json_path).unwrap();
    let parsed_json: Vec<Value> = serde_json::from_str(&json_content).unwrap();

    // Même nombre de records
    assert_eq!(parsed_jsonl.len(), parsed_json.len());
    assert_eq!(parsed_jsonl.len(), 3);

    // Mêmes données (par id)
    for (jl, j) in parsed_jsonl.iter().zip(parsed_json.iter()) {
        assert_eq!(jl["id"], j["id"], "Les IDs doivent correspondre");
        assert_eq!(jl["nom"], j["nom"], "Les noms doivent correspondre");
        // Les scores flottants peuvent avoir des différences minimes de représentation
        let score_jl = jl["score"].as_f64().unwrap();
        let score_j = j["score"].as_f64().unwrap();
        assert!((score_jl - score_j).abs() < 0.001, "Les scores doivent être cohérents");
    }
}

/// Test 13 : Robustesse avec des valeurs spéciales (null, vide, unicode)
#[test]
fn test_robustesse_valeurs_speciales() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("special.jsonl").to_str().unwrap().to_string();

    let mut writer = JsonLinesSinkWriter::new(&path).unwrap();

    // Record avec valeurs limites
    let record_avec_null = make_record(&[
        ("nom", json!("Étudiant ENSPD")),        // Unicode avec accents
        ("description", json!("Ligne 1\tTab")),   // Tabulation dans une valeur
        ("note", Value::Null),                     // Valeur nulle
        ("vide", json!("")),                       // Chaîne vide
        ("score", json!(0)),                       // Zéro
        ("negatif", json!(-5.5)),                  // Nombre négatif
        ("tres_grand", json!(9_999_999)),          // Grand nombre
    ]);

    writer.write_record(&record_avec_null).unwrap();
    writer.finalize().unwrap();

    let content = std::fs::read_to_string(&path).unwrap();
    let ligne = content.lines().next().unwrap();
    
    // Doit être du JSON valide
    let parsed: Value = serde_json::from_str(ligne)
        .expect("Les valeurs spéciales doivent produire du JSON valide");

    assert_eq!(parsed["note"], Value::Null);
    assert_eq!(parsed["score"], json!(0));
    assert_eq!(parsed["negatif"], json!(-5.5));
    // Les accents dans les valeurs string sont préservés
    assert!(parsed["nom"].as_str().unwrap().contains("ENSPD"));
}

/// Test 14 : Le writer JSONL est compatible avec l'interface SinkWriter (polymorphisme)
///
/// Ce test valide que JsonLinesSinkWriter peut être utilisé partout où un
/// `Box<dyn SinkWriter>` est attendu → le polymorphisme fonctionne.
#[test]
fn test_polymorphisme_sink_writer() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("poly.jsonl").to_str().unwrap().to_string();

    // On utilise le trait object Box<dyn SinkWriter>, pas le type concret
    let mut writer: Box<dyn SinkWriter> = Box::new(
        JsonLinesSinkWriter::new(&path).unwrap()
    );

    let record = make_record(&[("poly", json!("test_polymorphisme"))]);
    writer.write_record(&record).unwrap();
    writer.finalize().unwrap();

    let parsed = read_jsonl_file(&path);
    assert_eq!(parsed.len(), 1);
    assert_eq!(parsed[0]["poly"], json!("test_polymorphisme"));
}
