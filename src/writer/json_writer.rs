// src/writer/json_writer.rs
use super::{Record, SinkWriter};

pub struct JsonSinkWriter {
    // TODO: Ajouter les champs (NGLITANG #06)
    // records: Vec<Record>
    // output_path: String
}

impl SinkWriter for JsonSinkWriter {
    fn write_record(&mut self, record: &Record) -> anyhow::Result<()> {
        // TODO: Implémenter l'écriture JSON
        todo!("Implémenter l'écrivain JSON")
    }

    fn finalize(&mut self) -> anyhow::Result<()> {
        // TODO: Écrire tous les records en JSON
        todo!("Implémenter finalize JSON")
    }
}