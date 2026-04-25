use super::{Record, SourceReader};

pub struct JsonReader {
    pub path: String,
}

impl SourceReader for JsonReader {
    fn records(&self) -> Box<dyn Iterator<Item = anyhow::Result<Record>>> {
        // TODO: Implémenter la lecture JSON (DIOM #03)
        // 1. Ouvrir le fichier JSON
        // 2. Parser le tableau d'objets
        // 3. Convertir chaque objet en Record
        todo!("Implémenter le lecteur JSON")
    }
}

// TODO: Ajouter les tests unitaires (DIOM #03)