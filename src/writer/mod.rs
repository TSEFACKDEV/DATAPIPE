//! Module des écrivains (SinkWriter) pour DataPipe...
//! Définit le contrat que tous les formats de sortie doivent respecter.

use crate::reader::Record;
use anyhow::Result;

/// Contrat que tout écrivain de sortie doit implémenter.
pub trait SinkWriter {
    /// Écrit un seul enregistrement dans la sortie.
    /// Peut être appelé plusieurs fois (streaming).
    fn write_record(&mut self, record: &Record) -> Result<()>;

    /// Finalise l'écriture  (flush, fermeture, sérialisation finale).
    fn finalize(&mut self) -> Result<()>;
}
