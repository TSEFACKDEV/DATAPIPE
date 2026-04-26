//! Écrivain de fichiers CSV avec gestion automatique des en-têtes.

use crate::reader::Record;
use crate::writer::SinkWriter;
use anyhow::Result;
use csv::Writer;
use std::fs::File;
use std::io::BufWriter;

/// Écrivain CSV qui :
/// - écrit automatiquement la ligne d'en-tête à partir des clés du premier Record
/// - utilise BufWriter pour les performances
pub struct CsvSinkWriter {
    writer: Writer<BufWriter<File>>,
    headers_written: bool,
    headers: Vec<String>,
}

impl CsvSinkWriter {
    /// Crée un nouvel écrivain CSV.
    /// Délimiteur par défaut = ','.
    pub fn new(path: &str) -> Result<Self> {
        let file = File::create(path)?;
        let buf = BufWriter::new(file);
        let writer = Writer::from_writer(buf);
        Ok(CsvSinkWriter {
            writer,
            headers_written: false,
            headers: Vec::new(),
        })
    }
}

impl SinkWriter for CsvSinkWriter {
    fn write_record(&mut self, record: &Record) -> Result<()> {
        // Étape 1 : si pas encore d'en-têtes, les déduire des clés du premier record
        if !self.headers_written {
            self.headers = record.keys().cloned().collect();
            self.writer.write_record(&self.headers)?;
            self.headers_written = true;
        }

        // Étape 2 : construire une ligne CSV dans l'ordre des en-têtes
        let row: Vec<String> = self
            .headers
            .iter()
            .map(|h| {
                record
                    .get(h)
                    .map(|v| v.to_string())
                    .unwrap_or_default()
            })
            .collect();

        self.writer.write_record(&row)?;
        Ok(())
    }

    fn finalize(&mut self) -> Result<()> {
        self.writer.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reader::Record;
    use serde_json::json;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_csv_writer_simple() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("output.csv");
        let mut writer = CsvSinkWriter::new(path.to_str().unwrap()).unwrap();

        let mut rec1 = Record::new();
        rec1.insert("nom".to_string(), json!("Ruben"));
        rec1.insert("age".to_string(), json!(22));

        let mut rec2 = Record::new();
        rec2.insert("nom".to_string(), json!("Calvin"));
        rec2.insert("age".to_string(), json!(23));

        writer.write_record(&rec1).unwrap();
        writer.write_record(&rec2).unwrap();
        writer.finalize().unwrap();

        let content = fs::read_to_string(path).unwrap();
        assert!(content.contains("nom,age"));
        assert!(content.contains("Ruben,22"));
        assert!(content.contains("Calvin,23"));
    }
}
