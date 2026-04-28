#![allow(dead_code)]

use crate::reader::Record;
use std::collections::HashMap;
use serde_json::Value;

pub fn build_lookup(
    records: impl Iterator<Item = anyhow::Result<Record>>,
    key: &str,
) -> HashMap<String, Record> {
    let mut lookup = HashMap::new();
    for result in records {
        match result {
            Ok(record) => {
                if let Some(key_val) = record.get(key) {
                    let key_str = match key_val {
                        Value::String(s) => s.clone(),
                        other => other.to_string(),
                    };
                    lookup.insert(key_str, record);
                }
            }
            Err(e) => eprintln!("Erreur lookup: {}", e),
        }
    }
    lookup
}

pub fn join_records(
    mut left: Record,
    right_lookup: &HashMap<String, Record>,
    left_key: &str,
    join_type: &str,
) -> Option<Record> {
    let key_val = match left.get(left_key) {
        Some(Value::String(s)) => s.clone(),
        Some(other) => other.to_string(),
        None => {
            return if join_type == "left" { Some(left) } else { None };
        }
    };

    match right_lookup.get(&key_val) {
        Some(right_record) => {
            for (col, val) in right_record {
                left.entry(col.clone()).or_insert(val.clone());
            }
            Some(left)
        }
        None => {
            if join_type == "left" { Some(left) } else { None }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_record(pairs: &[(&str, &str)]) -> Record {
        pairs.iter().map(|(k, v)| (k.to_string(), json!(v))).collect()
    }

    #[test]
    fn test_build_lookup() {
        let records = vec![
            Ok(make_record(&[("id", "C01"), ("nom", "Jean")])),
            Ok(make_record(&[("id", "C02"), ("nom", "Marie")])),
        ];
        let lookup = build_lookup(records.into_iter(), "id");
        assert_eq!(lookup.len(), 2);
        assert!(lookup.contains_key("C01"));
        assert!(lookup.contains_key("C02"));
    }

    #[test]
    fn test_inner_join_found() {
        let left = make_record(&[("client_id", "C01"), ("montant", "45000")]);
        let mut lookup = HashMap::new();
        lookup.insert("C01".to_string(),
            make_record(&[("id", "C01"), ("nom", "Jean"), ("ville", "Douala")]));
        let result = join_records(left, &lookup, "client_id", "inner");
        assert!(result.is_some());
        let r = result.unwrap();
        assert_eq!(r.get("nom"), Some(&json!("Jean")));
        assert_eq!(r.get("montant"), Some(&json!("45000")));
    }

    #[test]
    fn test_inner_join_not_found() {
        let left = make_record(&[("client_id", "C99"), ("montant", "1000")]);
        let lookup: HashMap<String, Record> = HashMap::new();
        let result = join_records(left, &lookup, "client_id", "inner");
        assert!(result.is_none());
    }

    #[test]
    fn test_left_join_not_found() {
        let left = make_record(&[("client_id", "C99"), ("montant", "1000")]);
        let lookup: HashMap<String, Record> = HashMap::new();
        let result = join_records(left, &lookup, "client_id", "left");
        assert!(result.is_some());
        let r = result.unwrap();
        assert_eq!(r.get("montant"), Some(&json!("1000")));
    }
}
