pub mod csv_writer;
pub mod json_writer;
pub mod jsonl_writer;
pub mod factory;

use super::reader::Record;

/// Trait SinkWriter : contrat pour tous les écrivains
pub trait SinkWriter {
    /// Écrit un record dans la sortie
    fn write_record(&mut self, record: &Record) -> anyhow::Result<()>;
    
    /// Finalise l'écriture (flush)
    fn finalize(&mut self) -> anyhow::Result<()>;
}