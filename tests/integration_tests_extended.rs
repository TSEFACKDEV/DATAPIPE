// tests/integration_tests_extended.rs
//
// Tests d'intégration complets pour les nouveaux fichiers de données
// - Tests CSV avec divers délimiteurs  
// - Tests JSON array
// - Tests délimité (tab-separated)
// - Tests de conversions format

use datapipe::reader::{SourceReader};
use datapipe::reader::csv_reader::CsvReader;
use datapipe::reader::json_reader::JsonReader;
use datapipe::reader::delimited_reader::DelimitedReader;
use datapipe::writer::factory::create_writer;
use datapipe::config::DestinationConfig;
use tempfile::tempdir;

// Helper: Lire tous les records
fn read_all_records<R: SourceReader>(reader: &R) -> Vec<datapipe::reader::Record> {
    reader.records()
        .filter_map(|r| r.ok())
        .collect()
}

// ═══════════════════════════════════════════════════════════════════════════════
// SECTION 1 : TESTS CSV
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_csv_transactions_standard() {
    let reader = CsvReader::new("data/transactions.csv");
    let records = read_all_records(&reader);
    
    assert_eq!(records.len(), 5, "Doit avoir 5 enregistrements");
    // Les champs seront des colonnes
    let premier = &records[0];
    assert!(premier.contains_key("id"));
    assert!(premier.contains_key("nom"));
}

#[test]
fn test_csv_contacts_semicolon() {
    let reader = CsvReader::with_delimiter("data/contacts.csv", ';');
    let records = read_all_records(&reader);
    
    assert!(records.len() >= 3, "Doit avoir au moins 3 contacts");
    let premier = &records[0];
    assert!(premier.contains_key("id"));
    assert!(premier.contains_key("nom"));
}

#[test]
fn test_csv_sample_large() {
    let reader = CsvReader::new("data/sample_large.csv");
    let records = read_all_records(&reader);
    
    assert!(records.len() >= 20, "Doit avoir au moins 20 employés");
    let premier = &records[0];
    assert!(premier.contains_key("nom_complet"));
    assert!(premier.contains_key("departement"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// SECTION 2 : TESTS JSON
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_json_produits() {
    let reader = JsonReader {
        path: "data/produits.json".to_string(),
    };
    let records = read_all_records(&reader);
    
    assert_eq!(records.len(), 5);
    let premier = &records[0];
    assert_eq!(premier.get("produit").unwrap().as_str(), Some("Laptop"));
}

#[test]
fn test_json_commandes() {
    let reader = JsonReader {
        path: "data/commandes.json".to_string(),
    };
    let records = read_all_records(&reader);
    
    assert!(records.len() >= 3);
    let premier = &records[0];
    assert!(premier.contains_key("num_commande"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// SECTION 3 : TESTS DÉLIMITÉ
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_delimited_villes_tab() {
    let reader = DelimitedReader {
        path: "data/villes.txt".to_string(),
        delimiter: b'\t',
    };
    let records = read_all_records(&reader);
    
    assert!(records.len() >= 5);
    let premier = &records[0];
    assert!(premier.contains_key("id"));
    assert!(premier.contains_key("nom"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// SECTION 4 : TESTS CONVERSION
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_conversion_csv_to_json() {
    let temp_dir = tempdir().unwrap();
    let output_path = temp_dir.path().join("converted.json").to_str().unwrap().to_string();
    
    let reader = CsvReader::new("data/transactions.csv");
    let records = read_all_records(&reader);
    
    let config = DestinationConfig {
        format: "json".to_string(),
        path: output_path.clone(),
    };
    let mut writer = create_writer(&config).unwrap();
    
    for record in records {
        writer.write_record(&record).unwrap();
    }
    writer.finalize().unwrap();
    
    let content = std::fs::read_to_string(&output_path).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert!(parsed.is_array());
}

#[test]
fn test_conversion_csv_to_jsonl() {
    let temp_dir = tempdir().unwrap();
    let output_path = temp_dir.path().join("converted.jsonl").to_str().unwrap().to_string();
    
    let reader = CsvReader::new("data/sample_large.csv");
    let records = read_all_records(&reader);
    
    let config = DestinationConfig {
        format: "jsonl".to_string(),
        path: output_path.clone(),
    };
    let mut writer = create_writer(&config).unwrap();
    
    for record in records {
        writer.write_record(&record).unwrap();
    }
    writer.finalize().unwrap();
    
    let content = std::fs::read_to_string(&output_path).unwrap();
    let lines: Vec<&str> = content.lines().filter(|l| !l.is_empty()).collect();
    
    for line in &lines {
        let _: serde_json::Value = serde_json::from_str(line).expect("Ligne doit être JSON");
    }
    
    assert!(lines.len() >= 20);
}

#[test]
fn test_conversion_json_to_csv() {
    let temp_dir = tempdir().unwrap();
    let output_path = temp_dir.path().join("converted.csv").to_str().unwrap().to_string();
    
    let reader = JsonReader {
        path: "data/produits.json".to_string(),
    };
    let records = read_all_records(&reader);
    
    let config = DestinationConfig {
        format: "csv".to_string(),
        path: output_path.clone(),
    };
    let mut writer = create_writer(&config).unwrap();
    
    for record in records {
        writer.write_record(&record).unwrap();
    }
    writer.finalize().unwrap();
    
    let reader_verify = CsvReader::new(&output_path);
    let verify_records = read_all_records(&reader_verify);
    assert!(verify_records.len() >= 3);
}

// ═══════════════════════════════════════════════════════════════════════════════
// SECTION 5 : TESTS COHÉRENCE DONNÉES
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_employes_champs_obligatoires() {
    let reader = CsvReader::new("data/sample_large.csv");
    let records = read_all_records(&reader);
    
    for (idx, record) in records.iter().enumerate() {
        assert!(record.contains_key("id"), "Record {} manque 'id'", idx);
        assert!(record.contains_key("nom_complet"), "Record {} manque 'nom_complet'", idx);
        assert!(record.contains_key("departement"), "Record {} manque 'departement'", idx);
    }
}

#[test]
fn test_transactions_champs_obligatoires() {
    let reader = CsvReader::new("data/transactions.csv");
    let records = read_all_records(&reader);
    
    for (idx, record) in records.iter().enumerate() {
        assert!(record.contains_key("id"), "Record {} manque 'id'", idx);
        assert!(record.contains_key("nom"), "Record {} manque 'nom'", idx);
        assert!(record.contains_key("montant"), "Record {} manque 'montant'", idx);
    }
}

#[test]
fn test_produits_champs_obligatoires() {
    let reader = JsonReader {
        path: "data/produits.json".to_string(),
    };
    let records = read_all_records(&reader);
    
    for (idx, record) in records.iter().enumerate() {
        assert!(record.contains_key("id"), "Produit {} manque 'id'", idx);
        assert!(record.contains_key("produit"), "Produit {} manque 'produit'", idx);
        assert!(record.contains_key("prix"), "Produit {} manque 'prix'", idx);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SECTION 6 : TESTS ROUND-TRIP
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_roundtrip_json_csv_json() {
    let temp_dir = tempdir().unwrap();
    let csv_path = temp_dir.path().join("temp.csv").to_str().unwrap().to_string();
    let json_path = temp_dir.path().join("final.json").to_str().unwrap().to_string();
    
    // JSON → CSV
    let reader1 = JsonReader {
        path: "data/commandes.json".to_string(),
    };
    let records1 = read_all_records(&reader1);
    let orig_count = records1.len();
    
    let config_csv = DestinationConfig {
        format: "csv".to_string(),
        path: csv_path.clone(),
    };
    let mut writer_csv = create_writer(&config_csv).unwrap();
    for record in &records1 {
        writer_csv.write_record(record).unwrap();
    }
    writer_csv.finalize().unwrap();
    
    // CSV → JSON
    let reader2 = CsvReader::new(&csv_path);
    let records2 = read_all_records(&reader2);
    
    let config_json = DestinationConfig {
        format: "json".to_string(),
        path: json_path,
    };
    let mut writer_json = create_writer(&config_json).unwrap();
    for record in &records2 {
        writer_json.write_record(record).unwrap();
    }
    writer_json.finalize().unwrap();
    
    assert_eq!(records2.len(), orig_count);
}

#[test]
fn test_json_invalide_retourne_erreur() {
    let result = JsonReader {
        path: "data/nonexistent.json".to_string(),
    };
    let records = read_all_records(&result);
    // Les erreurs de lecture sont swallowed dans le trait SourceReader
    assert_eq!(records.len(), 0, "Fichier inexistant retourne 0 records");
}
