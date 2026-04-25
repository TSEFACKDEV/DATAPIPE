// src/validation.rs
use crate::reader::Record;
use crate::config::SchemaConfig;

pub fn validate_record(record: &Record, schema: &SchemaConfig) -> Vec<String> {
    // TODO: Implémenter la validation (DONFACK #08)
    let mut errors = Vec::new();
    
    // Vérifier les colonnes requises
    for col in &schema.required_columns {
        if !record.contains_key(col) {
            errors.push(format!("Colonne requise manquante: {}", col));
        }
    }
    
    // TODO: Vérifier les types si spécifiés
    if let Some(column_types) = &schema.column_types {
        // Vérification des types
    }
    
    errors
}