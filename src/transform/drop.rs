// =============================================================================
// FICHIER : src/transform/drop.rs
// AUTEUR  : NOLACK KAWUNJIBI FRANGE PARKER (#05)
// RÔLE    : Transformation de SUPPRESSION - retire une colonne d'un Record.
// =============================================================================
//
// ROLE DU FICHIER
// Dans les données brutes, il existe souvent des colonnes qu'on ne veut PAS
// dans le fichier de sortie :
//   - Colonnes sensibles : mot_de_passe, numero_carte, code_pin
//   - Colonnes techniques internes : id_interne, timestamp_db, flag_debug
//   - Colonnes redondantes : après un calcul, l'originale peut être superflue
//
// EXEMPLE D'UTILISATION dans pipeline.toml :
//   [[transforms]]
//   type = "drop"
//   column = "mot_de_passe"

// =============================================================================

// Imports depuis le module parent (transform/mod.rs)
use super::{Record, Transform};

// =============================================================================
// STRUCTURE DropTransform
// =============================================================================
pub struct DropTransform {
    /// Nom de la colonne à supprimer du Record
    pub column: String,
}

// =============================================================================
// IMPLÉMENTATION DU TRAIT Transform POUR DropTransform
// =============================================================================
impl Transform for DropTransform {
    // -------------------------------------------------------------------------
    // MÉTHODE : apply
    //
    // Logique :
    //   1. On enlève la colonne du record (si elle existe)
    //   2. On retourne le record sans cette colonne
    //
    //   - Ne regarde pas la valeur de la colonne, seulement son nom
    // -------------------------------------------------------------------------
    fn apply(&self, mut record: Record) -> Option<Record> {
        // `record.remove(&self.column)` :
        //   - Cherche la clé `self.column` dans la HashMap
        //   - Si trouvée : retire la paire clé/valeur et retourne Some(valeur_supprimee)
        //   - Si pas trouvée : ne fait rien et retourne None
        //
        // On ignore le résultat (valeur supprimée) avec `let _ = ...`
        // parce que ce qu'on a supprimé ne compte plus
        // L'underscore `_` est la convention Rust pour "variable ignorée intentionnellement".
        let _ = record.remove(&self.column);

        // On retourne le record modifié (sans la colonne supprimée).
        // `Some(record)` : le record continue dans le pipeline.
        Some(record)
    }

    fn name(&self) -> &str {
        "drop"
    }
}

// =============================================================================
// TESTS UNITAIRES
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::HashMap;

    /// Crée un Record de test avec plusieurs colonnes.
    fn make_record(paires: &[(&str, &str)]) -> Record {
        let mut record = HashMap::new();
        for (k, v) in paires {
            record.insert(k.to_string(), json!(v));
        }
        record
    }

    #[test]
    fn test_drop_supprime_colonne_existante() {
        let transform = DropTransform {
            column: "mot_de_passe".to_string(),
        };
        // Record avec 3 colonnes, dont "mot_de_passe" à supprimer
        let record = make_record(&[
            ("nom", "Jean"),
            ("email", "jean@example.com"),
            ("mot_de_passe", "secret123"),
        ]);

        let result = transform.apply(record).unwrap();

        // "mot_de_passe" doit avoir disparu
        assert!(!result.contains_key("mot_de_passe"), "La colonne doit être supprimée");
        // Les autres colonnes doivent être intactes
        assert_eq!(result["nom"], json!("Jean"));
        assert_eq!(result["email"], json!("jean@example.com"));
        // Vérifier que le nombre de colonnes a diminué de 1
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_drop_colonne_absente_est_silencieux() {
        // Si la colonne n'existe pas → pas d'erreur, le record passe intact
        let transform = DropTransform {
            column: "colonne_inexistante".to_string(),
        };
        let record = make_record(&[("nom", "Marie"), ("ville", "Yaoundé")]);
        let result = transform.apply(record);

        // Ne doit pas retourner None (ne filtre pas)
        assert!(result.is_some());
        // Le record original est intact
        let result = result.unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result["nom"], json!("Marie"));
    }

    #[test]
    fn test_drop_ne_filtre_jamais() {
        // Même sur un record vide, drop retourne Some (jamais None)
        let transform = DropTransform {
            column: "quelque_chose".to_string(),
        };
        let record = HashMap::new(); // Record complètement vide
        let result = transform.apply(record);
        assert!(result.is_some()); // Jamais None !
    }

    #[test]
    fn test_drop_plusieurs_colonnes_sensibles() {
        // Simulation : on chaîne plusieurs Drop pour supprimer plusieurs colonnes.
        // (Dans le pipeline, chaque transform est appliquée séquentiellement)
        let drop1 = DropTransform { column: "mot_de_passe".to_string() };
        let drop2 = DropTransform { column: "numero_carte".to_string() };

        let record = make_record(&[
            ("nom", "Alain"),
            ("montant", "50000"),
            ("mot_de_passe", "secret"),
            ("numero_carte", "1234-5678"),
        ]);

        // Application séquentielle (comme dans le pipeline)
        let r1 = drop1.apply(record).unwrap();
        let r2 = drop2.apply(r1).unwrap();

        assert!(!r2.contains_key("mot_de_passe"));
        assert!(!r2.contains_key("numero_carte"));
        assert_eq!(r2.len(), 2); // Il doit rester seulement "nom" et "montant"
    }
}
