use super::{Record, SourceReader};

pub struct DelimitedReader {
    pub path: String,
    pub delimiter: u8,
}

impl SourceReader for DelimitedReader {
    fn records(&self) -> Box<dyn Iterator<Item = anyhow::Result<Record>>> {
        // TODO: Implémenter la lecture de texte délimité (DIOM #03)
        // Similaire au CSV mais avec délimiteur personnalisable
        todo!("Implémenter le lecteur délimité")
    }
}