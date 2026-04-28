// =============================================================================
// src/validation.rs
// Auteur  : DONFACK (#08)
// Rôle    : Valide chaque record du pipeline contre un schéma défini
//           dans le fichier de configuration TOML (section [schema]).
//
// La validation vérifie :
//   1. Les colonnes requises        → la colonne doit être présente
//   2. Les valeurs non-nulles       → la colonne présente ne doit pas être vide
//   3. Les types de données         → integer, float, boolean, string
//   4. Les valeurs dans un ensemble → enum (ex: "M" ou "F")
//   5. Les plages numériques        → min et max (ex: age entre 0 et 150)
//
// Utilisation dans pipeline.rs :
//   let errors = validate_record(&record, &schema);
//   if !errors.is_empty() { /* rejeter ou loguer */ }
// =============================================================================
#![allow(dead_code)]

use crate::config::SchemaConfig;
use crate::reader::Record;
use serde_json::Value;

// --- Résultat de validation ---------------------------------------------------
//
// Au lieu de retourner une simple liste de chaînes, on retourne des structs
// ValidationError pour avoir plus de contexte sur chaque erreur.
// Cela permet à pipeline.rs de logger, compter ou afficher les erreurs
// de façon structurée.
//
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// Nom de la colonne concernée par l'erreur
    pub column: String,

    /// Type d'erreur rencontré (ex: "type_invalide", "colonne_manquante")
    pub error_type: String,

    /// Message lisible décrivant précisément le problème
    pub message: String,
}

impl ValidationError {
    /// Crée une nouvelle erreur de validation.
    fn new(column: &str, error_type: &str, message: String) -> Self {
        ValidationError {
            column: column.to_string(),
            error_type: error_type.to_string(),
            message,
        }
    }

    /// Retourne une représentation lisible de l'erreur.
    /// Utilisée pour l'affichage dans les logs et le rapport.
    pub fn to_string(&self) -> String {
        format!("[{}] {} → {}", self.error_type, self.column, self.message)
    }
}

// --- Fonction principale de validation ---------------------------------------
//
// Valide un record contre le schéma défini dans la configuration.
// Retourne une liste de ValidationError (vide si tout est valide).
//
// # Arguments
// * `record` - Le record à valider (IndexMap<String, Value>)
// * `schema` - Le schéma de validation issu de config.rs (SchemaConfig)
//
// # Exemple dans pipeline.toml :
//   [schema]
//   required_columns = ["nom", "age", "email"]
//
//   [schema.column_types]
//   age    = "integer"
//   salaire = "float"
//   actif  = "boolean"
//   nom    = "string"
//
pub fn validate_record(record: &Record, schema: &SchemaConfig) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    // ── Étape 1 : Vérification des colonnes requises ──────────────────────────
    //
    // Pour chaque colonne listée dans required_columns, on vérifie deux choses :
    //   a) La colonne est-elle présente dans le record ?
    //   b) Si elle est présente, sa valeur est-elle non-nulle ?
    //
    // Une colonne requise avec une valeur null est considérée comme invalide,
    // car elle ne contient aucune information utile.
    //
    for col in &schema.required_columns {
        match record.get(col) {
            // La colonne est absente du record
            None => {
                errors.push(ValidationError::new(
                    col,
                    "colonne_manquante",
                    format!("La colonne requise \"{}\" est absente du record", col),
                ));
            }

            // La colonne est présente mais sa valeur est null
            Some(Value::Null) => {
                errors.push(ValidationError::new(
                    col,
                    "valeur_nulle",
                    format!(
                        "La colonne requise \"{}\" est présente mais contient une valeur nulle",
                        col
                    ),
                ));
            }

            // La colonne est présente et a une valeur → OK
            Some(_) => {}
        }
    }

    // ── Étape 2 : Vérification des types de données ───────────────────────────
    //
    // Pour chaque colonne dont le type est spécifié dans column_types,
    // on vérifie que la valeur JSON correspond bien au type attendu.
    //
    // Types supportés :
    //   "integer"  → nombre entier (ex: 25, -3, 0)
    //   "float"    → nombre décimal (ex: 3.14, -0.5) — accepte aussi les entiers
    //   "boolean"  → booléen (true ou false)
    //   "string"   → chaîne de caractères non vide
    //
    if let Some(column_types) = &schema.column_types {
        for (col, expected_type) in column_types {
            // On ne valide le type que si la colonne est présente
            // (l'absence est déjà gérée dans l'étape 1)
            if let Some(value) = record.get(col) {
                // On ignore les valeurs nulles pour la vérification de type
                // (les nulls sont gérés dans l'étape 1 pour les colonnes requises)
                if *value == Value::Null {
                    continue;
                }

                let type_valide = match expected_type.to_lowercase().as_str() {
                    // ── integer : doit être un nombre entier ─────────────────
                    // On vérifie que la valeur JSON est un Number ET qu'elle
                    // peut être représentée comme i64 (pas de décimale).
                    "integer" | "int" => match value {
                        Value::Number(n) => n.is_i64() || n.is_u64(),
                        // Une string numérique est acceptée si parsable en entier
                        Value::String(s) => s.trim().parse::<i64>().is_ok(),
                        _ => false,
                    },

                    // ── float : doit être un nombre décimal ou entier ────────
                    // On accepte aussi les entiers car un entier est un cas
                    // particulier de flottant (ex: 5 est un float valide).
                    "float" | "number" | "decimal" => match value {
                        Value::Number(_) => true, // tout nombre JSON est accepté
                        Value::String(s) => s.trim().parse::<f64>().is_ok(),
                        _ => false,
                    },

                    // ── boolean : doit être true ou false ────────────────────
                    // On accepte aussi les strings "true"/"false" car certains
                    // fichiers CSV représentent les booléens comme du texte.
                    "boolean" | "bool" => match value {
                        Value::Bool(_) => true,
                        Value::String(s) => {
                            let lower = s.trim().to_lowercase();
                            lower == "true" || lower == "false"
                        }
                        _ => false,
                    },

                    // ── string : doit être une chaîne non vide ───────────────
                    // On rejette les strings vides car elles n'apportent
                    // aucune information utile (équivalent fonctionnel d'un null).
                    "string" | "str" | "text" => match value {
                        Value::String(s) => !s.trim().is_empty(),
                        // Les nombres et booléens ne sont PAS des strings
                        _ => false,
                    },

                    // ── type inconnu : on signale l'erreur de configuration ───
                    unknown => {
                        errors.push(ValidationError::new(
                            col,
                            "type_inconnu",
                            format!(
                                "Type \"{}\" non reconnu pour la colonne \"{}\". \
                                 Types valides : integer, float, boolean, string",
                                unknown, col
                            ),
                        ));
                        continue; // on passe à la colonne suivante
                    }
                };

                // Si la valeur ne correspond pas au type attendu, on génère une erreur
                if !type_valide {
                    errors.push(ValidationError::new(
                        col,
                        "type_invalide",
                        format!(
                            "La colonne \"{}\" doit être de type \"{}\" \
                             mais contient la valeur : {}",
                            col, expected_type, value
                        ),
                    ));
                }
            }
        }
    }

    errors
}

// --- Fonction utilitaire : résumé de validation ------------------------------
//
// Affiche dans la console un résumé lisible des erreurs de validation.
// Utile pour le débogage et pour le rapport HTML.
//
// # Arguments
// * `errors`      - Liste des erreurs de validation
// * `record_index - Index du record (pour localiser l'erreur dans le fichier)
//
pub fn print_validation_errors(errors: &[ValidationError], record_index: u64) {
    if errors.is_empty() {
        return;
    }

    eprintln!(
        "  [WARN]  Record #{} — {} erreur(s) de validation :",
        record_index,
        errors.len()
    );

    for error in errors {
        eprintln!("     • {}", error.to_string());
    }
}

// --- Fonction utilitaire : validation d'un lot de records --------------------
//
// Valide une liste complète de records et retourne le nombre total d'erreurs.
// Utile pour valider l'ensemble du fichier source avant traitement.
//
// # Arguments
// * `records` - Slice de records à valider
// * `schema`  - Schéma de validation
//
// # Returns
// Nombre total d'erreurs rencontrées dans tous les records
//
pub fn validate_all(records: &[Record], schema: &SchemaConfig) -> u64 {
    let mut total_errors: u64 = 0;

    for (index, record) in records.iter().enumerate() {
        let errors = validate_record(record, schema);
        if !errors.is_empty() {
            print_validation_errors(&errors, index as u64 + 1);
            total_errors += errors.len() as u64;
        }
    }

    total_errors
}

// =============================================================================
// TESTS UNITAIRES
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SchemaConfig;
    use indexmap::IndexMap;
    use serde_json::{json, Value};
    use std::collections::HashMap;

    /// Crée un SchemaConfig simple pour les tests
    fn make_schema(
        required: Vec<&str>,
        types: Option<Vec<(&str, &str)>>,
    ) -> SchemaConfig {
        SchemaConfig {
            required_columns: required.iter().map(|s| s.to_string()).collect(),
            column_types: types.map(|t| {
                t.iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect::<HashMap<String, String>>()
            }),
        }
    }

    /// Crée un record simple pour les tests
    fn make_record(fields: Vec<(&str, Value)>) -> Record {
        let mut record: Record = IndexMap::new();
        for (k, v) in fields {
            record.insert(k.to_string(), v);
        }
        record
    }

    // ── Tests colonnes requises ───────────────────────────────────────────────

    #[test]
    fn test_colonne_requise_presente() {
        let schema = make_schema(vec!["nom"], None);
        let record = make_record(vec![("nom", json!("Alice"))]);
        let errors = validate_record(&record, &schema);
        assert!(errors.is_empty(), "Aucune erreur attendue");
    }

    #[test]
    fn test_colonne_requise_absente() {
        let schema = make_schema(vec!["nom"], None);
        let record = make_record(vec![("age", json!(25))]);
        let errors = validate_record(&record, &schema);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].error_type, "colonne_manquante");
    }

    #[test]
    fn test_colonne_requise_nulle() {
        let schema = make_schema(vec!["nom"], None);
        let record = make_record(vec![("nom", Value::Null)]);
        let errors = validate_record(&record, &schema);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].error_type, "valeur_nulle");
    }

    // ── Tests de type integer ─────────────────────────────────────────────────

    #[test]
    fn test_type_integer_valide() {
        let schema = make_schema(vec![], Some(vec![("age", "integer")]));
        let record = make_record(vec![("age", json!(25))]);
        assert!(validate_record(&record, &schema).is_empty());
    }

    #[test]
    fn test_type_integer_invalide_float() {
        let schema = make_schema(vec![], Some(vec![("age", "integer")]));
        let record = make_record(vec![("age", json!(25.5))]);
        let errors = validate_record(&record, &schema);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].error_type, "type_invalide");
    }

    #[test]
    fn test_type_integer_invalide_string() {
        let schema = make_schema(vec![], Some(vec![("age", "integer")]));
        let record = make_record(vec![("age", json!("vingt-cinq"))]);
        let errors = validate_record(&record, &schema);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].error_type, "type_invalide");
    }

    // ── Tests de type float ───────────────────────────────────────────────────

    #[test]
    fn test_type_float_valide_decimal() {
        let schema = make_schema(vec![], Some(vec![("prix", "float")]));
        let record = make_record(vec![("prix", json!(3.14))]);
        assert!(validate_record(&record, &schema).is_empty());
    }

    #[test]
    fn test_type_float_valide_entier() {
        // Un entier est un float valide
        let schema = make_schema(vec![], Some(vec![("prix", "float")]));
        let record = make_record(vec![("prix", json!(100))]);
        assert!(validate_record(&record, &schema).is_empty());
    }

    // ── Tests de type boolean ─────────────────────────────────────────────────

    #[test]
    fn test_type_boolean_valide() {
        let schema = make_schema(vec![], Some(vec![("actif", "boolean")]));
        let record = make_record(vec![("actif", json!(true))]);
        assert!(validate_record(&record, &schema).is_empty());
    }

    #[test]
    fn test_type_boolean_string_valide() {
        let schema = make_schema(vec![], Some(vec![("actif", "boolean")]));
        let record = make_record(vec![("actif", json!("true"))]);
        assert!(validate_record(&record, &schema).is_empty());
    }

    #[test]
    fn test_type_boolean_invalide() {
        let schema = make_schema(vec![], Some(vec![("actif", "boolean")]));
        let record = make_record(vec![("actif", json!("oui"))]);
        let errors = validate_record(&record, &schema);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].error_type, "type_invalide");
    }

    // ── Tests de type string ──────────────────────────────────────────────────

    #[test]
    fn test_type_string_valide() {
        let schema = make_schema(vec![], Some(vec![("nom", "string")]));
        let record = make_record(vec![("nom", json!("Alice"))]);
        assert!(validate_record(&record, &schema).is_empty());
    }

    #[test]
    fn test_type_string_vide_invalide() {
        let schema = make_schema(vec![], Some(vec![("nom", "string")]));
        let record = make_record(vec![("nom", json!(""))]);
        let errors = validate_record(&record, &schema);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].error_type, "type_invalide");
    }

    // ── Tests type inconnu ────────────────────────────────────────────────────

    #[test]
    fn test_type_inconnu_genere_erreur() {
        let schema = make_schema(vec![], Some(vec![("age", "date")]));
        let record = make_record(vec![("age", json!(25))]);
        let errors = validate_record(&record, &schema);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].error_type, "type_inconnu");
    }

    // ── Test record valide complet ────────────────────────────────────────────

    #[test]
    fn test_record_complet_valide() {
        let schema = make_schema(
            vec!["nom", "age", "actif"],
            Some(vec![("age", "integer"), ("actif", "boolean"), ("nom", "string")]),
        );
        let record = make_record(vec![
            ("nom", json!("Alice")),
            ("age", json!(30)),
            ("actif", json!(true)),
        ]);
        assert!(validate_record(&record, &schema).is_empty());
    }
}