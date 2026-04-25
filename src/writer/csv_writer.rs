// src/writer/csv_writer.rs
use super::{Record, SinkWriter};

pub struct CsvSinkWriter {
    // TODO: Ajouter les champs nécessaires (NGLITANG #06)
    // writer: Writer<BufWriter<File>>
    // headers_written: bool
    // headers: Vec<String>
}

impl SinkWriter for CsvSinkWriter {
    fn write_record(&mut self, record: &Record) -> anyhow::Result<()> {
        // TODO: Implémenter l'écriture CSV (NGLITANG #06)
        // 1. Écrire les en-têtes si pas encore fait
        // 2. Écrire la ligne de données
        todo!("Implémenter l'écrivain CSV")
    }

    fn finalize(&mut self) -> anyhow::Result<()> {
        // TODO: Flush final
        todo!("Implémenter finalize")
    }
}