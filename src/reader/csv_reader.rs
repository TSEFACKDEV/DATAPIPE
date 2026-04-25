use super::{Record, SourceReader};
use serde_json::Value;

pub struct CsvReader {
    pub path: String,
    pub delimiter: u8,
}

impl SourceReader for CsvReader {
    fn records(&self) -> Box<dyn Iterator<Item = anyhow::Result<Record>>> {
        // TODO: Implémenter la lecture CSV (NZEUTEM #02)
        // 1. Ouvrir le fichier CSV
        // 2. Lire les en-têtes
        // 3. Pour chaque ligne, créer un Record
        // 4. Retourner un itérateur
        todo!("Implémenter le lecteur CSV")
    }
}

// TODO: Ajouter les tests unitaires (NZEUTEM #02)
#[cfg(test)]
mod tests {
    // Tests à implémenter
}