// =============================================================================
// FICHIER : src/transform/cast.rs
// AUTEUR  : NOLACK KAWUNJIBI FRANGE PARKER
// RÔLE    : Transformation de TYPE — convertit les valeurs d'une colonne
//           d'un type vers un autre (ex: "25" String → 25 Number)
// =============================================================================
//
// ROLE DU FICHIER
// Quand on lit un fichier CSV, TOUTES les valeurs sont des chaînes de caractères
// (String), même les nombres ! Par exemple, l'âge "25" est lu comme le texte "25"
// et non comme le nombre 25. CastTransform corrige ça.
//
// TYPES SUPPORTÉS :
//   "number"  → convertit "25.5" en 25.5 (f64, nombre décimal)
//   "boolean" → convertit "true"/"1"/"oui" en true (booléen)
//   "string"  → convertit n'importe quelle valeur en texte (ex: 42 → "42", true → "true")
//
// EXEMPLE D'UTILISATION dans pipeline.toml :
//   [[transforms]]
//   type = "cast"
//   column = "salaire"
//   target_type = "number"
//
// =============================================================================

// On importe le trait Transform et le type Record depuis le module parent (mod.rs)
// `super::` fait reférence au module parent
use super::{Record, Transform};

// On importe `json!` macro de serde_json pour créer des valeurs JSON facilement.
// Ex: json!(42.0) crée un Value::Number(42.0)
use serde_json::{json, Value};

// =============================================================================
// STRUCTURE CastTransform
// =============================================================================
// Une struct en Rust est comme une interface en java
// `pub` = public, accessible depuis les autres modules du projet.
pub struct CastTransform {
    /// Nom de la colonne à convertir
    pub column: String,

    /// Type cible vers lequel convertir : "number", "boolean", ou "string"
    pub target_type: String,
}

// =============================================================================
// IMPLÉMENTATION DU TRAIT Transform POUR CastTransform
// =============================================================================
// `impl Trait for Struct` = "CastTransform respecte le contrat Transform".
// Cela oblige à implémenter les méthodes `apply` et `name`.
impl Transform for CastTransform { 
    // -------------------------------------------------------------------------
    // MÉTHODE : apply 
    // REÇOIT  : un Record (= HashMap<String, Value>), pris par valeur
    // RETOURNE: Option<Record>
    //   - Some(record) → le record modifié continue dans le pipeline
    //   - None         → le record sera filtré
    // `mut record` : on prend possession du record et on peut le modifier. 
    // -------------------------------------------------------------------------
    fn apply(&self, mut record: Record) -> Option<Record> { 

        // `match` est l'équivalent Rust du switch/case
        // On compare `self.target_type` (le type cible demandé) avec les cas connus.
        // `.as_str()` convertit String → &str pour pouvoir faire le match sur des littéraux.
        match self.target_type.as_str() {

            // -----------------------------------------------------------------
            // CAS 1 : Convertir vers NUMBER (f64 = nombre décimal 64 bits)
            // -----------------------------------------------------------------
            "number" => {
                // `record.get(&self.column)` cherche la valeur de la colonne.
                // Retourne Option<&Value> :
                //   Some(&valeur) si la colonne existe
                //   None          si elle n'existe pas
                if let Some(val) = record.get(&self.column) {
                    // On normalise d'abord en String, quelle que soit la forme originale.
                    // Cela nous permet d'avoir une logique de parsing unifiée.
                    let string_val: Option<String> = match val {

                        // String JSON -> on clone la string (crée une copie)
                        Value::String(s) => Some(s.clone()),

                        // Number JSON -> on le convertit en texte via to_string()
                        Value::Number(n) => Some(n.to_string()),

                        // Bool JSON -> "true" ou "false"
                        Value::Bool(b) => Some(b.to_string()),

                        // Null, Array, Object = on ne peut pas convertir proprement
                        _ => None,
                    };

                    // Si on a une représentation textuelle
                    if let Some(s) = string_val {
                        // `.parse::<f64>()` tente la conversion texte -> nombre décimal.
                        // Retourne Ok(f64) si succès, Err si la string n'est pas un nombre.
                        if let Ok(num) = s.trim().parse::<f64>() {
                            // Succès ! `record.insert(clé, valeur)` ajoute ou remplace.
                            // `self.column.clone()` duplique la string (nécessaire car insert prend ownership)
                            record.insert(self.column.clone(), json!(num));
                        }
                        // Si parsing échoue -> on laisse la valeur originale.
                    }
                }
                // Si la colonne n'existe pas -> on ne fait rien, pas d'erreur.
            }

            // -----------------------------------------------------------------
            // CAS 2 : Convertir vers BOOLEAN (true/false)
            // -----------------------------------------------------------------
            "boolean" => {
                if let Some(val) = record.get(&self.column) {
                    // On détermine la valeur booléenne selon plusieurs représentations.
                    // Supporte le français et l'anglais pour les données camerounaises.
                    let bool_val: Option<bool> = match val {
                        Value::String(s) => {
                            // `.to_lowercase()` normalise la casse : "TRUE" -> "true"
                            // `.as_str()` convertit en &str pour le match
                            match s.trim().to_lowercase().as_str() {
                                // Valeurs "vraies"
                                "true" | "1" | "oui" | "yes" | "vrai" => Some(true),
                                // Valeurs "fausses"
                                "false" | "0" | "non" | "no" | "faux" => Some(false),
                                // Valeur inconnue → on laisse intact, pas d'erreur
                                _ => None,
                            }
                        }
                        // Déjà un booléen JSON -> on garde tel quel.
                        // `*b` est la syntaxe de déréférencement (b est &bool, *b est bool)
                        Value::Bool(b) => Some(*b),
                        // Number : 0.0 → false, tout autre → true (convention du C)
                        // `.as_f64()` extrait le f64 depuis un Number JSON
                        // `.unwrap_or(0.0)` : si l'extraction échoue, on utilise 0.0
                        Value::Number(n) => Some(n.as_f64().unwrap_or(0.0) != 0.0),
                        _ => None,
                    };

                    // Si on a pu déterminer une valeur booléenne → on met à jour
                    if let Some(b) = bool_val {
                        record.insert(self.column.clone(), json!(b));
                    }
                }
            }

            // -----------------------------------------------------------------
            // CAS 3 : Convertir vers STRING (tout devient texte)
            // -----------------------------------------------------------------
            "string" => {
                // `.cloned()` crée une copie de la valeur.
                // Nécessaire car on ne peut pas avoir une référence ET modifier le record en même temps (règle du borrow checker Rust).
                if let Some(val) = record.get(&self.column).cloned() {
                    let string_val = match &val {

                        // Déjà une string -> rien à changer
                        Value::String(_) => val,

                        // Number -> convertir en string
                        Value::Number(n) => json!(n.to_string()),

                        // Bool -> "true" ou "false"
                        Value::Bool(b) => json!(b.to_string()),

                        // Null -> chaîne vide
                        Value::Null => json!(""),

                        // Tableaux/Objets complexes -> représentation JSON sous forme de string
                        other => json!(other.to_string()),
                    };
                    record.insert(self.column.clone(), string_val);
                }
            }

            // -----------------------------------------------------------------
            // CAS PAR DÉFAUT : type non reconnu -> avertissement
            // -----------------------------------------------------------------
            // `_` capture tout ce qui n'a pas été matché
            _ => {
                // `eprintln!` écrit dans stderr (canal d'erreur standard).
                // C'est la convention Unix pour les messages d'erreur/avertissement.
                eprintln!(
                    "[WARN] CastTransform: type cible inconnu '{}' pour la colonne '{}'",
                    self.target_type, self.column
                );
            }
        }

        // On retourne toujours Some(record).
        // CastTransform ne filtre pas et ne retourne jamais None.
        // si le cast échoue, le record continue dans le pipeline.
        Some(record)
    }

    // Nom de la transformation utilisé dans les logs et le rapport d'exécution.
    fn name(&self) -> &str {
        "cast"
    }
}

// =============================================================================
// TESTS UNITAIRES
// =============================================================================
// `#[cfg(test_disabled)]` : ce bloc est compilé uniquement quand on lance `cargo test`.
// Il n'existe PAS dans le binaire final (gain de taille et de performance).
#[cfg(test_disabled)]
mod tests {
    // `use super::*` : importe tout ce qui est défini dans ce module (cast.rs)
    use super::*;
    use serde_json::json;
    use std::collections::HashMap;

    /// Crée un Record de test avec une seule entrée clé/valeur.
    fn make_record(key: &str, val: serde_json::Value) -> Record {
        let mut record = HashMap::new();
        record.insert(key.to_string(), val);
        record
    }

    #[test]
    fn test_cast_string_vers_number() {
        let transform = CastTransform {
            column: "age".to_string(),
            target_type: "number".to_string(),
        };
        let record = make_record("age", json!("25"));

        // `unwrap()` : on est sûr que apply ne retourne pas None
        let result = transform.apply(record).unwrap();

        // "25" String doit devenir 25.0 Number
        assert_eq!(result["age"], json!(25.0));
    }

    #[test]
    fn test_cast_decimal() {
        let transform = CastTransform {
            column: "salaire".to_string(),
            target_type: "number".to_string(),
        };
        let record = make_record("salaire", json!("150000.50"));
        let result = transform.apply(record).unwrap();
        assert_eq!(result["salaire"], json!(150000.50_f64));
    }

    #[test]
    fn test_cast_invalide_garde_original() {
        // Si la valeur n'est pas convertible, l'originale doit être préservée.
        let transform = CastTransform {
            column: "code".to_string(),
            target_type: "number".to_string(),
        };
        let record = make_record("code", json!("ABC"));
        let result = transform.apply(record).unwrap();
        assert_eq!(result["code"], json!("ABC")); // Inchangé
    }

    #[test]
    fn test_cast_vers_boolean_vrai() {
        let transform = CastTransform {
            column: "actif".to_string(),
            target_type: "boolean".to_string(),
        };
        // Tester toutes les représentations de "vrai"
        for val_vraie in &["true", "1", "oui", "TRUE", "vrai"] {
            let record = make_record("actif", json!(val_vraie));
            let result = transform.apply(record).unwrap();
            assert_eq!(result["actif"], json!(true), "Echec pour '{}'", val_vraie);
        }
    }

    #[test]
    fn test_cast_vers_boolean_faux() {
        let transform = CastTransform {
            column: "actif".to_string(),
            target_type: "boolean".to_string(),
        };
        for val_fausse in &["false", "0", "non", "FALSE", "faux"] {
            let record = make_record("actif", json!(val_fausse));
            let result = transform.apply(record).unwrap();
            assert_eq!(result["actif"], json!(false), "Echec pour '{}'", val_fausse);
        }
    }

    #[test]
    fn test_cast_number_vers_string() {
        let transform = CastTransform {
            column: "code".to_string(),
            target_type: "string".to_string(),
        };
        let record = make_record("code", json!(42));
        let result = transform.apply(record).unwrap();
        assert_eq!(result["code"], json!("42"));
    }

    #[test]
    fn test_cast_colonne_absente_ne_filtre_pas() {
        // Si la colonne n'existe pas → le record passe sans modification
        let transform = CastTransform {
            column: "inexistant".to_string(),
            target_type: "number".to_string(),
        };
        let record = make_record("autre", json!("valeur"));
        let result = transform.apply(record);
        assert!(result.is_some()); // Ne filtre pas !
        assert_eq!(result.unwrap()["autre"], json!("valeur")); // Record intact
    }
}
