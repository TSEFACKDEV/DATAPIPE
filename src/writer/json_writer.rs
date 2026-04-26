//! Écrivain JSON : accumule tous les records en mémoire,
//! puis écrit un tableau complet à la fin.

use crate::reader::Record;
use crate::writer::SinkWriter;
use anyhow::Result;
use serde_json::to_writer_pretty;
use std::fs::File;
use std::io::BufWriter;

/// Écrivain JSON standard : écrit un tableau d'objets [...]
/// Attention : accumule TOUT en RAM → bon pour petits fichiers.
pub struct JsonSinkWriter {
    records: Vec<Record>,
    output_path: String,
}

impl JsonSinkWriter {
    pub fn new(path: &str) -> Self {
        JsonSinkWriter {
            records: Vec::new(),
            output_path: path.to_string(),
        }
    }
}

impl SinkWriter for JsonSinkWriter {
    fn write_record(&mut self, record: &Record) -> Result<()> {
        self.records.push(record.clone());
        Ok(())
    }

    fn finalize(&mut self) -> Result<()> {
        let file = File::create(&self.output_path)?;
        let writer = BufWriter::new(file);
        to_writer_pretty(writer, &self.records)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_json_writer_simple() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("output.json");
        let mut writer = JsonSinkWriter::new(path.to_str().unwrap());

        let mut rec = Record::new();
        rec.insert("nom".to_string(), json!("Ruben"));
        rec.insert("role".to_string(), json!("dev"));

        writer.write_record(&rec).unwrap();
        writer.finalize().unwrap();

        let content = fs::read_to_string(path).unwrap();
        assert!(content.contains("Ruben"));
        assert!(content.contains("dev"));
        assert!(content.starts_with('['));
        assert!(content.ends_with(']'));
    }
}
