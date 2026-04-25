// src/transform/drop.rs
use super::{Record, Transform};

pub struct DropTransform {
    pub column: String,
}

impl Transform for DropTransform {
    fn apply(&self, mut record: Record) -> Option<Record> {
        // TODO: Implémenter la suppression de colonne (NOLACK #05)
        // 1. Supprimer la colonne spécifiée
        // 2. Retourner le record modifié
        todo!("Implémenter drop")
    }

    fn name(&self) -> &str {
        "drop"
    }
}