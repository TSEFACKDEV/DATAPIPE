use super::{Record, Transform};

pub struct FilterTransform {
    pub column: String,
    pub value: String,
    pub operator: String, // "=", "!=", "<", ">"
}

impl Transform for FilterTransform {
    fn apply(&self, record: Record) -> Option<Record> {
        // TODO: Implémenter le filtrage (ASSONGUE #04)
        // 1. Récupérer la valeur de la colonne
        // 2. Comparer selon l'opérateur
        // 3. Retourner Some(record) ou None
        todo!("Implémenter filter")
    }

    fn name(&self) -> &str {
        "filter"
    }
}