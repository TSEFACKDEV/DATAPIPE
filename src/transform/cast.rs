use super::{Record, Transform};

pub struct CastTransform {
    pub column: String,
    pub target_type: String, // "string", "number", "boolean"
}

impl Transform for CastTransform {
    fn apply(&self, mut record: Record) -> Option<Record> {
        // TODO: Implémenter le cast (NOLACK #05)
        // 1. Récupérer la valeur actuelle
        // 2. Convertir vers le type cible
        // 3. Mettre à jour le record
        todo!("Implémenter cast")
    }

    fn name(&self) -> &str {
        "cast"
    }
}