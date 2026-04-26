use super::{Record, SourceReader};
use serde_json::Value;
use csv::ReaderBuilder;
use std::fs::File;
use anyhow::{anyhow, Result};

pub struct DelimitedReader {
    pub path: String,
    pub delimiter: u8,
}

impl SourceReader for DelimitedReader {
    fn records(&self) -> Box<dyn Iterator<Item = anyhow::Result<Record>>> {
        // Charger les records du fichier délimité
        match load_delimited_records(&self.path, self.delimiter) {
            Ok(records) => Box::new(records.into_iter().map(Ok)),
            Err(e) => {
                // Retourner un itérateur qui produit l'erreur une fois
                Box::new(vec![Err(e)].into_iter())
            }
        }
    }
}

/// Charge un fichier texte délimité et retourne les records
fn load_delimited_records(path: &str, delimiter: u8) -> Result<Vec<Record>> {
    // 1️⃣ Ouvrir le fichier
    let file = File::open(path)
        .map_err(|e| anyhow!("Impossible de lire {}: {}", path, e))?;

    // 2️⃣ Créer un lecteur CSV avec le délimiteur spécifié
    let mut reader = ReaderBuilder::new()
        .delimiter(delimiter)
        .from_reader(file);

    // 3️⃣ Lire les en-têtes
    let headers = reader.headers()
        .map_err(|e| anyhow!("Erreur lecture en-têtes de {}: {}", path, e))?
        .clone();

    let header_names: Vec<String> = headers.iter().map(|s| s.to_string()).collect();

    // 4️⃣ Convertir chaque ligne en Record
    let mut records = Vec::new();
    
    for (line_num, result) in reader.records().enumerate() {
        let record = result
            .map_err(|e| anyhow!("Erreur ligne {} dans {}: {}", line_num + 2, path, e))?;

        // Créer un HashMap avec les en-têtes et les valeurs
        let mut map = Record::new();
        
        for (header, value) in header_names.iter().zip(record.iter()) {
            // Convertir la valeur en serde_json::Value (String)
            map.insert(header.clone(), Value::String(value.to_string()));
        }

        records.push(map);
    }

    Ok(records)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_delimited_reader_semicolon() {
        // Créer un fichier TSV (tabulation) temporaire
        let content = "name;age;city\nAlice;30;Paris\nBob;25;Lyon";
        
        let mut tmp_file = NamedTempFile::new().unwrap();
        tmp_file.write_all(content.as_bytes()).unwrap();
        tmp_file.flush().unwrap();

        let reader = DelimitedReader {
            path: tmp_file.path().to_string_lossy().to_string(),
            delimiter: b';',
        };

        let records: Vec<_> = reader.records()
            .filter_map(|r| r.ok())
            .collect();

        assert_eq!(records.len(), 2);
        assert_eq!(records[0]["name"], Value::String("Alice".to_string()));
        assert_eq!(records[0]["age"], Value::String("30".to_string()));
        assert_eq!(records[1]["city"], Value::String("Lyon".to_string()));
    }

    #[test]
    fn test_delimited_reader_tab() {
        // Créer un fichier TSV (tabulation) temporaire
        let content = "name\tage\tcity\nAlice\t30\tParis\nBob\t25\tLyon";
        
        let mut tmp_file = NamedTempFile::new().unwrap();
        tmp_file.write_all(content.as_bytes()).unwrap();
        tmp_file.flush().unwrap();

        let reader = DelimitedReader {
            path: tmp_file.path().to_string_lossy().to_string(),
            delimiter: b'\t',
        };

        let records: Vec<_> = reader.records()
            .filter_map(|r| r.ok())
            .collect();

        assert_eq!(records.len(), 2);
        assert_eq!(records[0]["name"], Value::String("Alice".to_string()));
    }

    #[test]
    fn test_delimited_reader_pipe() {
        // Créer un fichier délimité par pipe (|)
        let content = "name|dept|salary\nAlice|IT|5000\nBob|HR|4000";
        
        let mut tmp_file = NamedTempFile::new().unwrap();
        tmp_file.write_all(content.as_bytes()).unwrap();
        tmp_file.flush().unwrap();

        let reader = DelimitedReader {
            path: tmp_file.path().to_string_lossy().to_string(),
            delimiter: b'|',
        };

        let records: Vec<_> = reader.records()
            .filter_map(|r| r.ok())
            .collect();

        assert_eq!(records.len(), 2);
        assert_eq!(records[0]["dept"], Value::String("IT".to_string()));
    }

    #[test]
    fn test_delimited_reader_invalid_file() {
        let reader = DelimitedReader {
            path: "/nonexistent/file.txt".to_string(),
            delimiter: b',',
        };

        let results: Vec<_> = reader.records().collect();
        assert!(results[0].is_err());
    }
}