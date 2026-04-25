use super::{Record, Transform};

pub struct ComputeTransform {
    pub new_column: String,
    pub expression: String, // ex: "salaire * 0.1"
}

impl Transform for ComputeTransform {
    fn apply(&self, mut record: Record) -> Option<Record> {
        // TODO: Implémenter le calcul (NOLACK #05)
        // 1. Parser l'expression
        // 2. Évaluer avec les valeurs du record
        // 3. Ajouter la nouvelle colonne
        todo!("Implémenter compute")
    }

    fn name(&self) -> &str {
        "compute"
    }
}