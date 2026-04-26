// =============================================================================
// FICHIER : src/transform/factory.rs
// AUTEUR  : NOLACK KAWUNJIBI FRANGE PARKER (#05)
// RÔLE    : Fabrique de transformations
//           Lit une Transform Config (données du TOML) et crée la bonne
//           transformation concrète.
// =============================================================================
//
// ROLE DU FICHIER
// L'orchestrateur (pipeline.rs) ne sait pas QUI créer. Il lit la config TOML
// et obtient des Transform Config avec un champ `type = "cast"` etc.
// C'est la FACTORY qui fait la traduction :
//   TransformConfig { type: "cast",    column: "age", target_type: "number" } -> create_transform()
//   Box<dyn Transform>  [CastTransform { column: "age", target_type: "number" }]
//
// DESIGN PATTERN "Factory" :
// Ce pattern permet de créer des objets sans que le code appelant
// connaisse les types concrets. L'orchestrateur manipule Box<dyn Transform>
// (trait object), il est complètement découplé des implémentations concrètes.
//
// `Box<dyn Transform>` = pointeur sur le tas (heap) vers quelque chose qui
// implémente Transform. Le type exact n'est connu qu'à l'exécution (dynamic dispatch).
//
// EXEMPLE D'UTILISATION dans pipeline.rs :
//   let transforms: Vec<Box<dyn Transform>> = config.transforms
//       .iter()
//       .map(|tc| create_transform(tc))
//       .collect();
//
// =============================================================================

// On importe le trait Transform et tous les types concrets de transformations.
// `super::` = module parent (transform/mod.rs)
// La double mention `super::rename::RenameTransform` = "dans le module parent,
// dans le sous-module rename, prends RenameTransform"
use super::{
    Record,
    Transform,
    rename::RenameTransform,
    filter::FilterTransform,
    cast::CastTransform,
    compute::ComputeTransform,
    drop::DropTransform,
};

// On importe Transform Config depuis le module config du crate.
// `crate::` = depuis la racine du projet (src/main.rs ou src/lib.rs)
use crate::config::TransformConfig;

// =============================================================================
// FONCTION PRINCIPALE : create_transform
// =============================================================================
// Crée et retourne une transformation concrète à partir de sa configuration.
//
// PARAMÈTRE  : &TransformConfig — référence vers la config lue depuis le TOML
// RETOURNE   : Box<dyn Transform> — pointeur vers n'importe quelle transformation
//
// `Box<dyn Transform>` est nécessaire car :
//   - Les traits objects n'ont pas de taille connue à la compilation
//   - `Box` alloue la mémoire sur le tas (heap), ce qui résout ce problème
//   - `dyn` indique le dispatch dynamique (la vraie méthode est trouvée au runtime)
pub fn create_transform(config: &TransformConfig) -> Box<dyn Transform> {
    // `config.r#type` : le `r#` est nécessaire car `type` est un MOT-CLÉ RÉSERVÉ en Rust.
    // On ne peut pas écrire `config.type` directement, il faut l'échapper avec r#.
    // `.as_str()` convertit le String en &str pour le match.
    match config.r#type.as_str() {

        // -----------------------------------------------------------------
        // CAS "rename" : Renommage de colonne
        // -----------------------------------------------------------------
        // from: colonne source (obligatoire pour rename)
        // to:   nouveau nom   (obligatoire pour rename)
        "rename" => Box::new(RenameTransform {
            // `config.from.clone()` : clone la String Option<String> et la déballe.
            // `.unwrap_or_default()` : si `from` est None -> String vide "" (évite le crash).
            // En production on voudrait retourner une Err, mais ici on est permissifs.
            from: config.from.clone().unwrap_or_default(),
            to:   config.to.clone().unwrap_or_default(),
        }),

        // -----------------------------------------------------------------
        // CAS "filter" : Filtrage de records par valeur
        // -----------------------------------------------------------------
        // column:   colonne à vérifier (obligatoire)
        // value:    valeur à comparer  (obligatoire)
        // operator: opérateur de comparaison, "=" par défaut si absent
        "filter" => Box::new(FilterTransform {
            column:   config.column.clone().unwrap_or_default(),
            value:    config.value.clone().unwrap_or_default(),
            // `unwrap_or_else(|| "=".to_string())` :
            //   Si `operator` est None -> on utilise "=" comme opérateur par défaut.
            //   C'est plus sûr que unwrap_or("=".to_string()) car la closure n'est
            //   évaluée que si nécessaire (lazy evaluation).
            operator: config.operator.clone().unwrap_or_else(|| "=".to_string()),
        }),

        // -----------------------------------------------------------------
        // CAS "cast" : Conversion de type
        // -----------------------------------------------------------------
        // column:      colonne à convertir (obligatoire)
        // target_type: type cible : "number", "boolean", "string" (obligatoire)
        "cast" => Box::new(CastTransform {
            column:      config.column.clone().unwrap_or_default(),
            // "string" est le type par défaut si target_type est absent dans le TOML
            target_type: config.target_type.clone().unwrap_or_else(|| "string".to_string()),
        }),

        // -----------------------------------------------------------------
        // CAS "compute" : Calcul de nouvelle colonne
        // -----------------------------------------------------------------
        // new_column:  nom de la colonne à créer (obligatoire)
        // expression:  formule de calcul (obligatoire) ex: "salaire * 0.1"
        "compute" => Box::new(ComputeTransform {
            new_column: config.new_column.clone().unwrap_or_default(),
            expression: config.expression.clone().unwrap_or_default(),
        }),

        // -----------------------------------------------------------------
        // CAS "drop" : Suppression de colonne
        // -----------------------------------------------------------------
        // column: colonne à supprimer (obligatoire)
        "drop" => Box::new(DropTransform {
            column: config.column.clone().unwrap_or_default(),
        }),

        // -----------------------------------------------------------------
        // CAS PAR DÉFAUT : type de transformation inconnu
        // -----------------------------------------------------------------
        // Si le TOML contient `type = "teleportation"` par exemple,
        // on crée une transformation "no-op" (qui ne fait rien) plutôt que de crasher.
        // C'est plus robuste qu'un panic! pour un outil ETL en production.
        unknown_type => {
            // `eprintln!` écrit dans stderr (canal d'erreur standard).
            eprintln!(
                "[WARN] create_transform: type de transformation inconnu '{}'. \
                 Types supportés: rename, filter, cast, compute, drop. \
                 Une transformation identité (no-op) sera utilisée.",
                unknown_type
            );
            // On retourne une transformation "identité" qui laisse tout passer intact.
            // `NoOpTransform` est défini juste en dessous dans ce fichier.
            Box::new(NoOpTransform { type_name: unknown_type.to_string() })
        }
    }
}

// =============================================================================
// STRUCTURE PRIVÉE : NoOpTransform
// =============================================================================
// "No Operation" Transform : ne fait rien, laisse les records passer intacts.
// Utilisée comme fallback pour les types inconnus.
// `pub(super)` = visible uniquement dans ce module et son parent.
struct NoOpTransform {
    type_name: String, // Pour pouvoir afficher un log utile
}

impl Transform for NoOpTransform {
    fn apply(&self, record: Record) -> Option<Record> {
        // Passe le record sans modification
        Some(record)
    }
    fn name(&self) -> &str {
        // Rustique : on pourrait retourner self.type_name.as_str() mais
        // on ne peut pas retourner une référence vers une String locale.
        // On retourne un nom générique.
        "noop"
    }
}

// (Record est disponible via le trait Transform importé ci-dessus)

// =============================================================================
// TESTS UNITAIRES
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::HashMap;

    // Crée une TransformConfig minimale pour les tests.
    // `..Default::default()` remplit tous les champs non spécifiés avec leur valeur par défaut.
    // Nécessite que TransformConfig implémente Default, ce qui est le cas grâce à serde.
    fn make_config(type_name: &str) -> TransformConfig {
        TransformConfig {
            r#type: type_name.to_string(),
            from: None,
            to: None,
            column: None,
            value: None,
            target_type: None,
            new_column: None,
            expression: None,
            operator: None,
        }
    }

    /// Crée un record de test simple
    fn make_record(key: &str, val: serde_json::Value) -> HashMap<String, serde_json::Value> {
        let mut r = HashMap::new();
        r.insert(key.to_string(), val);
        r
    }

    #[test]
    fn test_factory_cree_rename() {
        let mut config = make_config("rename");
        config.from = Some("ancien".to_string());
        config.to   = Some("nouveau".to_string());

        let transform = create_transform(&config);
        // Vérifier que c'est bien un "rename" par son nom
        assert_eq!(transform.name(), "rename");
    }

    #[test]
    fn test_factory_cree_cast() {
        let mut config = make_config("cast");
        config.column      = Some("age".to_string());
        config.target_type = Some("number".to_string());

        let transform = create_transform(&config);
        assert_eq!(transform.name(), "cast");

        // Vérifier que la transformation fonctionne effectivement
        let record = make_record("age", json!("30"));
        let result = transform.apply(record).unwrap();
        assert_eq!(result["age"], json!(30.0_f64));
    }

    #[test]
    fn test_factory_cree_compute() {
        let mut config = make_config("compute");
        config.new_column  = Some("double".to_string());
        config.expression  = Some("valeur * 2".to_string());

        let transform = create_transform(&config);
        assert_eq!(transform.name(), "compute");

        let record = make_record("valeur", json!(50.0));
        let result = transform.apply(record).unwrap();
        assert_eq!(result["double"], json!(100.0_f64));
    }

    #[test]
    fn test_factory_cree_drop() {
        let mut config = make_config("drop");
        config.column = Some("secret".to_string());

        let transform = create_transform(&config);
        assert_eq!(transform.name(), "drop");

        let record = make_record("secret", json!("motdepasse"));
        let result = transform.apply(record).unwrap();
        assert!(!result.contains_key("secret"));
    }

    #[test]
    fn test_factory_type_inconnu_retourne_noop() {
        // Un type inconnu doit retourner une transformation no-op (pas de panic!)
        let config = make_config("transformation_qui_nexiste_pas");
        let transform = create_transform(&config);

        // La transformation no-op doit laisser passer les records intacts
        let record = make_record("col", json!("valeur"));
        let result = transform.apply(record);
        assert!(result.is_some());
        assert_eq!(result.unwrap()["col"], json!("valeur"));
    }

    #[test]
    fn test_factory_tous_les_types() {
        // Vérifier que tous les types supportés sont reconnus
        let types_supportes = vec!["rename", "filter", "cast", "compute", "drop"];
        for type_name in types_supportes {
            let config = make_config(type_name);
            // Ceci ne doit pas paniquer
            let _transform = create_transform(&config);
            // Si on arrive ici sans crash, le type est bien géré
        }
    }
}
