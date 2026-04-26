// src/transform/mod.rs
// [ASSONGUE] - Trait Transform + type Record
//
// Ce module définit le contrat que TOUTES les transformations doivent respecter.
// Les autres membres de l'équipe (NOLACK, etc.) implémenteront leur propre struct
// qui implémente ce trait.

use std::collections::HashMap;

// ─────────────────────────────────────────────
//  Type central : un enregistrement = une ligne
// ─────────────────────────────────────────────

/// Un enregistrement est une map ordonnable clé → valeur.
/// Toutes les valeurs sont stockées en String ; les transformations
/// de type (cast) se chargeront de les convertir si nécessaire.
///
/// Exemple : {"nom": "Alice", "age": "30", "ville": "Paris"}
pub type Record = HashMap<String, String>;

// ─────────────────────────────────────────────
//  Trait Transform
// ─────────────────────────────────────────────

/// Contrat que doit respecter chaque transformation du pipeline.
///
/// # Règle de retour
/// - `Some(record)` → l'enregistrement est conservé (éventuellement modifié)
/// - `None`         → l'enregistrement est **supprimé** du flux (filtré)
///
/// # Exemple d'implémentation minimale
/// ```rust
/// use datapipe::transform::{Transform, Record};
///
/// struct NoOp;
///
/// impl Transform for NoOp {
///     fn apply(&self, record: Record) -> Option<Record> {
///         Some(record) // ne fait rien, laisse passer
///     }
///     fn name(&self) -> &str {
///         "noop"
///     }
/// }
/// ```
pub trait Transform: Send + Sync {
    /// Applique la transformation sur un enregistrement.
    ///
    /// Prend possession du `record` pour permettre des modifications
    /// in-place sans clonage inutile.
    fn apply(&self, record: Record) -> Option<Record>;

    /// Nom lisible de la transformation (utilisé dans les logs et rapports).
    fn name(&self) -> &str;
}

// ─────────────────────────────────────────────
//  Re-exports publics des sous-modules
// ─────────────────────────────────────────────

pub mod filter;
pub mod rename;

pub use filter::FilterTransform;
pub use rename::RenameTransform;

// ─────────────────────────────────────────────
//  Utilitaire : appliquer une chaîne de transforms
// ─────────────────────────────────────────────

/// Applique une liste de transformations séquentiellement sur un enregistrement.
///
/// Dès qu'une transformation retourne `None`, le traitement s'arrête
/// et `None` est propagé (court-circuit).
///
/// # Arguments
/// * `record`     – l'enregistrement source
/// * `transforms` – tranche de boxed transforms à appliquer dans l'ordre
///
/// # Retour
/// `Some(record_transformé)` ou `None` si filtré en cours de route.
pub fn apply_chain(
    record: Record,
    transforms: &[Box<dyn Transform>],
) -> Option<Record> {
    transforms.iter().fold(Some(record), |acc, t| {
        acc.and_then(|r| t.apply(r))
    })
}

// ─────────────────────────────────────────────
//  Tests du module racine
// ─────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // Transform identité pour tester apply_chain
    struct Identity;
    impl Transform for Identity {
        fn apply(&self, record: Record) -> Option<Record> { Some(record) }
        fn name(&self) -> &str { "identity" }
    }

    // Transform qui filtre tout
    struct DropAll;
    impl Transform for DropAll {
        fn apply(&self, _record: Record) -> Option<Record> { None }
        fn name(&self) -> &str { "drop_all" }
    }

    fn make_record(pairs: &[(&str, &str)]) -> Record {
        pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
    }

    #[test]
    fn apply_chain_vide_retourne_record_intact() {
        let record = make_record(&[("a", "1")]);
        let transforms: Vec<Box<dyn Transform>> = vec![];
        let result = apply_chain(record.clone(), &transforms);
        assert_eq!(result, Some(record));
    }

    #[test]
    fn apply_chain_identity_retourne_record_intact() {
        let record = make_record(&[("x", "hello")]);
        let transforms: Vec<Box<dyn Transform>> = vec![Box::new(Identity)];
        let result = apply_chain(record.clone(), &transforms);
        assert_eq!(result, Some(record));
    }

    #[test]
    fn apply_chain_drop_all_retourne_none() {
        let record = make_record(&[("x", "hello")]);
        let transforms: Vec<Box<dyn Transform>> = vec![
            Box::new(Identity),
            Box::new(DropAll),
            Box::new(Identity), // ne doit jamais être atteint
        ];
        let result = apply_chain(record, &transforms);
        assert_eq!(result, None);
    }

    #[test]
    fn apply_chain_court_circuite_apres_none() {
        // DropAll suivi d'Identity : le résultat doit rester None
        let record = make_record(&[("x", "1")]);
        let transforms: Vec<Box<dyn Transform>> = vec![
            Box::new(DropAll),
            Box::new(Identity),
        ];
        assert_eq!(apply_chain(record, &transforms), None);
    }
}
