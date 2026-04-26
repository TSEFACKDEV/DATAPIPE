use super::{Record, SourceReader};
use serde_json::Value;
use std::fs;
use anyhow::{anyhow, Result};

pub struct JsonReader {
    pub path: String,
}

impl SourceReader for JsonReader {
    fn records(&self) -> Box<dyn Iterator<Item = anyhow::Result<Record>>> {
        // Charger et parser le fichier JSON
        match load_json_records(&self.path) {
            Ok(records) => Box::new(records.into_iter().map(Ok)),
            Err(e) => {
                // Retourner un itérateur qui produit l'erreur une fois
                Box::new(vec![Err(e)].into_iter())
            }
        }
    }
}

/// Charge un fichier JSON et retourne les records
fn load_json_records(path: &str) -> Result<Vec<Record>> {
    // 1️⃣ Lire le fichier
    let content = fs::read_to_string(path)
        .map_err(|e| anyhow!("Impossible de lire {}: {}", path, e))?;

    // 2️⃣ Parser le JSON
    let json_value: Value = serde_json::from_str(&content)
        .map_err(|e| anyhow!("Erreur de parsing JSON dans {}: {}", path, e))?;

    // 3️⃣ S'assurer que c'est un array
    let array = json_value.as_array()
        .ok_or_else(|| anyhow!("Le fichier JSON doit contenir un array"))?;

    // 4️⃣ Convertir chaque élément en Record
    let mut records = Vec::new();
    for (idx, item) in array.iter().enumerate() {
        let obj = item.as_object()
            .ok_or_else(|| anyhow!("Élément {} n'est pas un objet JSON", idx))?;

        // Convertir l'objet en HashMap<String, Value>
        let record: Record = obj.iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        records.push(record);
    }

    Ok(records)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_json_reader_simple() {
        // Créer un fichier JSON temporaire
        let json_content = r#"[
            {"name": "Alice", "age": 30},
            {"name": "Bob", "age": 25}
        ]"#;

        let mut tmp_file = NamedTempFile::new().unwrap();
        tmp_file.write_all(json_content.as_bytes()).unwrap();
        tmp_file.flush().unwrap();

        let reader = JsonReader {
            path: tmp_file.path().to_string_lossy().to_string(),
        };

        let records: Vec<_> = reader.records()
            .filter_map(|r| r.ok())
            .collect();

        assert_eq!(records.len(), 2);
        assert_eq!(records[0]["name"], Value::String("Alice".to_string()));
        assert_eq!(records[1]["age"], Value::Number(25u32.into()));
    }

    #[test]
    fn test_json_reader_invalid_file() {
        let reader = JsonReader {
            path: "/nonexistent/file.json".to_string(),
        };

        let results: Vec<_> = reader.records().collect();
        assert!(results[0].is_err());
    }

    #[test]
    fn test_json_reader_not_array() {
        let json_content = r#"{"name": "Alice"}"#;
        
        let mut tmp_file = NamedTempFile::new().unwrap();
        tmp_file.write_all(json_content.as_bytes()).unwrap();
        tmp_file.flush().unwrap();

        let reader = JsonReader {
            path: tmp_file.path().to_string_lossy().to_string(),
        };

        let results: Vec<_> = reader.records().collect();
        assert!(results[0].is_err());
    }
}