use super::{Record, Transform};

pub struct RenameTransform {
    pub from: String,
    pub to: String,
}

impl Transform for RenameTransform {
    fn apply(&self, mut record: Record) -> Option<Record> {
        // TODO: Implémenter le renommage (ASSONGUE #04)
        // 1. Vérifier si la colonne 'from' existe
        // 2. Supprimer l'ancienne clé
        // 3. Insérer avec le nouveau nom
        todo!("Implémenter rename")
    }

    fn name(&self) -> &str {
        "rename"
    }
}