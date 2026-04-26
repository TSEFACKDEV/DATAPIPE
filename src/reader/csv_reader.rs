// =============================================================================
// src/reader/csv_reader.rs
// Auteure : NZEUTEM DOMMOE Eunice Felixtine (#02)
// Rôle    : Lit un fichier CSV et convertit chaque ligne en Record.
//           Supporte les délimiteurs personnalisés (virgule, point-virgule, tab…)
// =============================================================================

// On importe les types dont on a besoin
use crate::reader::{Record, SourceReader}; // notre contrat et notre type
use anyhow::Context;                           // gestion d'erreurs ergonomique
use csv::ReaderBuilder;                    // la bibliothèque csv de Rust
use serde_json::{json, Value};             // pour créer des Value JSON facilement
use std::fs::File;                         // pour ouvrir un fichier

// --- La structure CsvReader --------------------------------------------------
//
// Une structure en Rust = un objet avec des champs.
// CsvReader a besoin de deux informations pour fonctionner :
//   - path      : le chemin vers le fichier CSV à lire
//   - delimiter : le caractère qui sépare les colonnes (souvent ',' ou ';' ou '\t')
//
#[derive(Debug)]
pub struct CsvReader {
    pub path: String,
    pub delimiter: char,
}

// --- Constructeur ------------------------------------------------------------
//
// new() est la façon idiomatique de créer un CsvReader en Rust.
// On fournit un délimiteur par défaut (virgule) si non précisé.
//
impl CsvReader {
    /// Crée un nouveau CsvReader avec délimiteur virgule par défaut.
    pub fn new(path: &str) -> Self {
        CsvReader {
            path: path.to_string(),
            delimiter: ',',
        }
    }

    /// Crée un CsvReader avec un délimiteur personnalisé.
    /// Exemples : ';' pour les fichiers français, '\t' pour les TSV
    pub fn with_delimiter(path: &str, delimiter: char) -> Self {
        CsvReader {
            path: path.to_string(),
            delimiter,
        }
    }
}

// --- Implémentation du trait SourceReader ------------------------------------
//
// Ici on "respecte le contrat" : on implémente la méthode records().
// Cette méthode retourne un itérateur sur les lignes du CSV.
//
impl SourceReader for CsvReader {
    fn records(&self) -> Box<dyn Iterator<Item = anyhow::Result<Record>>> {
        // --- Étape 1 : Ouvrir le fichier ---
        // On utilise unwrap_or_else pour donner un message d'erreur clair.
        // En production on gérerait l'erreur plus proprement, mais ici c'est
        // suffisant pour le projet.
        let file = match File::open(&self.path) {
            Ok(f) => f,
            Err(e) => {
                // Si le fichier n'existe pas, on retourne un itérateur
                // qui contient une seule erreur.
                let msg = format!("Impossible d'ouvrir '{}': {}", self.path, e);
                return Box::new(std::iter::once(Err(anyhow::anyhow!(msg))));
            }
        };

        // --- Étape 2 : Configurer le lecteur CSV ---
        // ReaderBuilder permet de personnaliser le comportement du lecteur.
        // On convertit le char en u8 car la bibliothèque csv attend un octet.
        let delimiter_byte = self.delimiter as u8;

        let mut rdr = ReaderBuilder::new()
            .delimiter(delimiter_byte)        // le séparateur de colonnes
            .has_headers(true)                // la première ligne = en-têtes
            .trim(csv::Trim::All)             // supprimer les espaces autour des valeurs
            .flexible(true)                   // tolérer les lignes avec colonnes manquantes
            .from_reader(file);

        // --- Étape 3 : Lire les en-têtes (noms de colonnes) ---
        // La première ligne du CSV contient les noms des colonnes.
        // On les lit UNE FOIS et on les clone pour pouvoir les utiliser dans l'itérateur.
        let headers = match rdr.headers() {
            Ok(h) => h.clone(),
            Err(e) => {
                let msg = format!("Erreur lecture en-têtes '{}': {}", self.path, e);
                return Box::new(std::iter::once(Err(anyhow::anyhow!(msg))));
            }
        };

        // On clone les en-têtes une deuxième fois pour les capturer dans la closure.
        // En Rust, quand une closure capture une variable, elle doit souvent en prendre
        // possession (move). Donc on a besoin d'un clone qu'on peut déplacer.
        let headers_for_closure = headers.clone();

        // --- Étape 4 : Construire l'itérateur ---
        // rdr.into_records() consomme le lecteur et retourne un itérateur
        // sur les lignes (sans les en-têtes).
        //
        // Pour chaque ligne, on crée un Record (IndexMap) en associant
        // chaque en-tête à la valeur correspondante.
        //
        // La closure avec "move" prend possession de headers_for_closure
        // pour qu'elle reste disponible pendant toute la durée de l'itération.
        let iter = rdr.into_records().map(move |result| {
            // result est soit Ok(StringRecord) soit Err(csv::Error)
            let row = result.context("Erreur lors de la lecture d'une ligne CSV")?;

            // On construit le Record en zippant en-têtes et valeurs.
            // zip() assemble deux itérateurs en paires : (en-tête, valeur)
            let record: Record = headers_for_closure
                .iter()
                .zip(row.iter())
                .map(|(header, valeur)| {
                    // On essaie de convertir la valeur en nombre si possible,
                    // sinon on la garde comme texte.
                    let json_value = parse_value(valeur);
                    (header.to_string(), json_value)
                })
                .collect();

            Ok(record)
        });

        Box::new(iter)
    }
}

// --- Fonction utilitaire : parse_value ---------------------------------------
//
// Tente de deviner le type d'une valeur CSV :
//   - Si c'est un entier  → Value::Number (ex: 25)
//   - Si c'est un décimal → Value::Number (ex: 3.14)
//   - Si c'est "true"/"false" → Value::Bool
//   - Si c'est vide       → Value::Null
//   - Sinon               → Value::String (texte)
//
// Note : dans un vrai pipeline, on préfère garder tout en String et laisser
// la transformation Cast décider des types. Mais ici on fait une détection
// automatique légère pour être plus pratique.
//
fn parse_value(s: &str) -> Value {
    let trimmed = s.trim();

    // Valeur vide → null
    if trimmed.is_empty() {
        return Value::Null;
    }

    // Booléen
    if trimmed.eq_ignore_ascii_case("true") {
        return Value::Bool(true);
    }
    if trimmed.eq_ignore_ascii_case("false") {
        return Value::Bool(false);
    }

    // Entier (ex: 42, -7)
    if let Ok(n) = trimmed.parse::<i64>() {
        return json!(n);
    }

    // Décimal (ex: 3.14, -0.5)
    if let Ok(f) = trimmed.parse::<f64>() {
        return json!(f);
    }

    // Par défaut : texte
    Value::String(trimmed.to_string())
}

// =============================================================================
// TESTS UNITAIRES
// Ces tests sont compilés et exécutés SEULEMENT quand on fait `cargo test`.
// Ils sont complètement ignorés dans la version release finale.
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile; // on utilisera un fichier temporaire

    // --- Fonction helper : crée un fichier CSV temporaire --------------------
    // Cette fonction est utilisée par tous les tests. Elle crée un vrai fichier
    // sur le disque, y écrit le contenu donné, et retourne le fichier.
    // Le fichier est automatiquement supprimé quand il sort de portée.
    fn creer_csv_temp(contenu: &str) -> NamedTempFile {
        let mut f = NamedTempFile::new().expect("Impossible de créer fichier temp");
        write!(f, "{}", contenu).expect("Impossible d'écrire dans fichier temp");
        f
    }

    // -------------------------------------------------------------------------
    // TEST 1 : Lecture basique d'un CSV simple avec virgule
    // -------------------------------------------------------------------------
    #[test]
    fn test_lecture_csv_simple() {
        let contenu = "nom,age,ville\nJean,25,Douala\nMarie,30,Yaoundé";
        let fichier = creer_csv_temp(contenu);
        let chemin = fichier.path().to_str().unwrap();

        let lecteur = CsvReader::new(chemin);
        let records: Vec<_> = lecteur.records().collect();

        // On doit avoir exactement 2 lignes (les en-têtes ne comptent pas)
        assert_eq!(records.len(), 2, "Devrait avoir 2 records");

        // Vérifier la première ligne
        let ligne1 = records[0].as_ref().unwrap();
        assert_eq!(ligne1["nom"], Value::String("Jean".to_string()));
        assert_eq!(ligne1["age"], json!(25));        // 25 est reconnu comme nombre
        assert_eq!(ligne1["ville"], Value::String("Douala".to_string()));

        // Vérifier la deuxième ligne
        let ligne2 = records[1].as_ref().unwrap();
        assert_eq!(ligne2["nom"], Value::String("Marie".to_string()));
        assert_eq!(ligne2["age"], json!(30));
        assert_eq!(ligne2["ville"], Value::String("Yaoundé".to_string()));
    }

    // -------------------------------------------------------------------------
    // TEST 2 : Données camerounaises réalistes — étudiants ENSP
    // -------------------------------------------------------------------------
    #[test]
    fn test_donnees_etudiants_camerounais() {
        let contenu = "matricule,nom_complet,filiere,moyenne,statut\n\
            22G00347,NZEUTEM Eunice,Génie Informatique,14.5,Admis\n\
            22G00540,TSEFACK Calvin,Génie Informatique,16.2,Admis\n\
            22G00071,DIOM Lucraine,Génie Logiciel,11.8,Admis\n\
            22G00335,NOLACK Frange,Génie Informatique,9.5,Ajourné";

        let fichier = creer_csv_temp(contenu);
        let lecteur = CsvReader::new(fichier.path().to_str().unwrap());
        let records: Vec<_> = lecteur.records().map(|r| r.unwrap()).collect();

        assert_eq!(records.len(), 4);

        // Vérifier NZEUTEM (première ligne = notre auteure !)
        assert_eq!(records[0]["matricule"], Value::String("22G00347".to_string()));
        assert_eq!(records[0]["nom_complet"], Value::String("NZEUTEM Eunice".to_string()));
        assert_eq!(records[0]["moyenne"], json!(14.5)); // nombre décimal détecté
        assert_eq!(records[0]["statut"], Value::String("Admis".to_string()));

        // Vérifier que l'ajourné est bien là aussi
        assert_eq!(records[3]["statut"], Value::String("Ajourné".to_string()));
        assert_eq!(records[3]["moyenne"], json!(9.5));
    }

    // -------------------------------------------------------------------------
    // TEST 3 : Délimiteur point-virgule (format français courant)
    // -------------------------------------------------------------------------
    #[test]
    fn test_delimiteur_point_virgule() {
        // Beaucoup de fichiers CSV français utilisent ; au lieu de ,
        // (car la virgule est utilisée pour les décimales en français)
        let contenu = "produit;prix_fcfa;quantite\n\
            Tomate;500;10\n\
            Poisson;2500;3\n\
            Plantain;300;20";

        let fichier = creer_csv_temp(contenu);
        let lecteur = CsvReader::with_delimiter(fichier.path().to_str().unwrap(), ';');
        let records: Vec<_> = lecteur.records().map(|r| r.unwrap()).collect();

        assert_eq!(records.len(), 3);
        assert_eq!(records[0]["produit"], Value::String("Tomate".to_string()));
        assert_eq!(records[0]["prix_fcfa"], json!(500)); // nombre reconnu
        assert_eq!(records[1]["produit"], Value::String("Poisson".to_string()));
        assert_eq!(records[1]["prix_fcfa"], json!(2500));
    }

    // -------------------------------------------------------------------------
    // TEST 4 : Délimiteur tabulation (format TSV)
    // -------------------------------------------------------------------------
    #[test]
    fn test_delimiteur_tabulation() {
        let contenu = "region\tpopulation\tcapitale\n\
            Centre\t4100000\tYaoundé\n\
            Littoral\t3800000\tDouala\n\
            Ouest\t2000000\tBafoussam";

        let fichier = creer_csv_temp(contenu);
        let lecteur = CsvReader::with_delimiter(fichier.path().to_str().unwrap(), '\t');
        let records: Vec<_> = lecteur.records().map(|r| r.unwrap()).collect();

        assert_eq!(records.len(), 3);
        assert_eq!(records[0]["region"], Value::String("Centre".to_string()));
        assert_eq!(records[0]["capitale"], Value::String("Yaoundé".to_string()));
        assert_eq!(records[0]["population"], json!(4100000));
    }

    // -------------------------------------------------------------------------
    // TEST 5 : Valeurs vides → doivent devenir null
    // -------------------------------------------------------------------------
    #[test]
    fn test_valeurs_vides_deviennent_null() {
        let contenu = "nom,telephone,email\n\
            Paul,699001122,paul@gmail.com\n\
            Alice,,alice@yahoo.fr\n\
            Bob,677334455,";

        let fichier = creer_csv_temp(contenu);
        let lecteur = CsvReader::new(fichier.path().to_str().unwrap());
        let records: Vec<_> = lecteur.records().map(|r| r.unwrap()).collect();

        // Alice n'a pas de téléphone → null
        assert_eq!(records[1]["telephone"], Value::Null);
        // Bob n'a pas d'email → null
        assert_eq!(records[2]["email"], Value::Null);
    }

    // -------------------------------------------------------------------------
    // TEST 6 : Valeurs booléennes reconnues
    // -------------------------------------------------------------------------
    #[test]
    fn test_valeurs_booleennes() {
        let contenu = "nom,actif,verifie\n\
            Test1,true,false\n\
            Test2,True,False\n\
            Test3,TRUE,FALSE";

        let fichier = creer_csv_temp(contenu);
        let lecteur = CsvReader::new(fichier.path().to_str().unwrap());
        let records: Vec<_> = lecteur.records().map(|r| r.unwrap()).collect();

        // true/false en toutes casses doivent être reconnus comme booléens
        assert_eq!(records[0]["actif"], Value::Bool(true));
        assert_eq!(records[0]["verifie"], Value::Bool(false));
        assert_eq!(records[1]["actif"], Value::Bool(true));
        assert_eq!(records[2]["verifie"], Value::Bool(false));
    }

    // -------------------------------------------------------------------------
    // TEST 7 : Fichier inexistant → erreur propre (pas de crash !)
    // -------------------------------------------------------------------------
    #[test]
    fn test_fichier_inexistant_erreur_propre() {
        let lecteur = CsvReader::new("/fichier/qui/nexiste/pas.csv");
        let records: Vec<_> = lecteur.records().collect();

        // On doit avoir exactement une erreur
        assert_eq!(records.len(), 1);
        // Ce doit être une erreur (pas un Ok)
        assert!(records[0].is_err());
    }

    // -------------------------------------------------------------------------
    // TEST 8 : CSV avec espaces autour des valeurs → doivent être nettoyés
    // -------------------------------------------------------------------------
    #[test]
    fn test_espaces_autour_valeurs_nettoyes() {
        let contenu = "nom , age , ville\n  Jean  ,  25  ,  Douala  ";

        let fichier = creer_csv_temp(contenu);
        let lecteur = CsvReader::new(fichier.path().to_str().unwrap());
        let records: Vec<_> = lecteur.records().map(|r| r.unwrap()).collect();

        assert_eq!(records.len(), 1);
        // Les espaces doivent être supprimés
        assert_eq!(records[0]["nom"], Value::String("Jean".to_string()));
        assert_eq!(records[0]["age"], json!(25));
    }

    // -------------------------------------------------------------------------
    // TEST 9 : CSV vide (seulement les en-têtes) → 0 records
    // -------------------------------------------------------------------------
    #[test]
    fn test_csv_vide_zero_records() {
        let contenu = "nom,age,ville\n"; // seulement les en-têtes

        let fichier = creer_csv_temp(contenu);
        let lecteur = CsvReader::new(fichier.path().to_str().unwrap());
        let records: Vec<_> = lecteur.records().collect();

        assert_eq!(records.len(), 0, "Un CSV vide doit retourner 0 records");
    }

    // -------------------------------------------------------------------------
    // TEST 10 : Données de marché — cas réaliste PME camerounaise
    // -------------------------------------------------------------------------
    #[test]
    fn test_donnees_marche_camerounais() {
        let contenu = "vendeur,produit,quantite_kg,prix_unitaire_fcfa,date_vente\n\
            Mama Ngono,Tomate,15,600,2024-03-15\n\
            Papa Bello,Macabo,50,400,2024-03-15\n\
            Tante Rose,Gombo,8,1200,2024-03-16\n\
            Oncle Pierre,Plantain,200,250,2024-03-16";

        let fichier = creer_csv_temp(contenu);
        let lecteur = CsvReader::new(fichier.path().to_str().unwrap());
        let records: Vec<_> = lecteur.records().map(|r| r.unwrap()).collect();

        assert_eq!(records.len(), 4);

        // Vérifier les types détectés
        assert_eq!(records[0]["vendeur"], Value::String("Mama Ngono".to_string()));
        assert_eq!(records[0]["quantite_kg"], json!(15));           // entier
        assert_eq!(records[0]["prix_unitaire_fcfa"], json!(600));   // entier
        // La date reste une chaîne (pas un type JSON spécial)
        assert_eq!(records[0]["date_vente"], Value::String("2024-03-15".to_string()));
    }

    // -------------------------------------------------------------------------
    // TEST 11 : Vérifier que l'ordre des colonnes est préservé (IndexMap)
    // -------------------------------------------------------------------------
    #[test]
    fn test_ordre_colonnes_preserve() {
        let contenu = "premier,deuxieme,troisieme\nA,B,C";

        let fichier = creer_csv_temp(contenu);
        let lecteur = CsvReader::new(fichier.path().to_str().unwrap());
        let records: Vec<_> = lecteur.records().map(|r| r.unwrap()).collect();

        let colonnes: Vec<&String> = records[0].keys().collect();
        assert_eq!(colonnes[0], "premier");
        assert_eq!(colonnes[1], "deuxieme");
        assert_eq!(colonnes[2], "troisieme");
    }
}
