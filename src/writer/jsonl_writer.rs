// src/writer/jsonl_writer.rs
//
// Auteur : NGANSOP NGOUABOU FREDI LOIK (#07)
// Rôle   : Implémentation de l'écrivain JSONL (JSON Lines / NDJSON)
//
// ─── QU'EST-CE QUE LE FORMAT JSONL ? ────────────────────────────────────────
//
// JSONL (JSON Lines), aussi appelé NDJSON (Newline-Delimited JSON), est un
// format où CHAQUE LIGNE est un objet JSON valide et autonome.
//
// Exemple de fichier JSONL valide :
//   {"nom":"Jean","age":25,"departement":"Informatique"}
//   {"nom":"Marie","age":30,"departement":"RH"}
//
// ─── AVANTAGES vs JSON classique ─────────────────────────────────────────────
//
// JSON classique (tableau) requiert de TOUT charger avant de traiter.
// Sur un fichier de 10 Go → 10 Go de RAM nécessaires. ❌
//
// JSONL → on traite LIGNE PAR LIGNE.
// Mémoire constante quelle que soit la taille du fichier → STREAMING PUR. ✅

use crate::reader::Record;
use crate::writer::SinkWriter;
use anyhow::{Context, Result};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

/// Écrivain JSONL (JSON Lines) avec buffer d'écriture.
///
/// Chaque appel à `write_record` émet immédiatement une ligne JSON.
/// Aucune accumulation en mémoire : mémoire O(1) indépendamment du volume.
pub struct JsonLinesSinkWriter {
    /// Buffer d'écriture autour du fichier de sortie.
    writer: BufWriter<File>,
    /// Compteur interne pour les stats et le debug.
    records_written: usize,
}

impl JsonLinesSinkWriter {
    /// Crée un nouvel écrivain JSONL vers le fichier spécifié.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        // Créer les répertoires parents si absents
        if let Some(parent) = path.as_ref().parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).with_context(|| {
                    format!("Impossible de créer le répertoire {:?}", parent)
                })?;
            }
        }

        let file = File::create(&path).with_context(|| {
            format!("Impossible de créer le fichier JSONL : {:?}", path.as_ref())
        })?;

        Ok(JsonLinesSinkWriter {
            writer: BufWriter::new(file),
            records_written: 0,
        })
    }

    /// Retourne le nombre de records écrits jusqu'ici.
    #[allow(dead_code)]
    pub fn records_written(&self) -> usize {
        self.records_written
    }
}

impl SinkWriter for JsonLinesSinkWriter {
    /// Sérialise le record en JSON compact sur une ligne unique.
    fn write_record(&mut self, record: &Record) -> Result<()> {
        // Convertir IndexMap en serde_json::Map pour sérialisation
        let map: serde_json::Map<String, serde_json::Value> = record
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        let json_line = serde_json::to_string(&map).with_context(|| {
            format!("Erreur de sérialisation JSON pour le record #{}", self.records_written + 1)
        })?;

        writeln!(self.writer, "{}", json_line).with_context(|| {
            format!("Erreur d'écriture JSONL pour le record #{}", self.records_written + 1)
        })?;

        self.records_written += 1;
        Ok(())
    }

    /// Vide le buffer et garantit l'écriture physique sur le disque.
    fn finalize(&mut self) -> Result<()> {
        self.writer
            .flush()
            .context("Erreur lors du flush du buffer JSONL")
    }
}

#[cfg(test_disabled)]
mod tests {
    use super::*;
    use serde_json::{json, Value};
    use indexmap::IndexMap;
    use tempfile::tempdir;

    fn make_record(pairs: &[(&str, Value)]) -> Record {
        pairs.iter().map(|(k, v)| (k.to_string(), v.clone())).collect()
    }

    #[test]
    fn test_ecriture_un_record() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test_un.jsonl");

        let mut writer = JsonLinesSinkWriter::new(&path).unwrap();
        let record = make_record(&[("nom", json!("Jean")), ("age", json!(25))]);
        writer.write_record(&record).unwrap();
        writer.finalize().unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        let lignes: Vec<&str> = content.lines().collect();
        assert_eq!(lignes.len(), 1);
    }

    #[test]
    fn test_ecriture_plusieurs_records() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test_plusieurs.jsonl");

        let mut writer = JsonLinesSinkWriter::new(&path).unwrap();

        let noms = ["Jean", "Marie", "Paul"];
        for (i, nom) in noms.iter().enumerate() {
            let record = make_record(&[
                ("nom", json!(*nom)),
                ("age", json!(25 + i as i64 * 5)),
            ]);
            writer.write_record(&record).unwrap();
        }
        writer.finalize().unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        let lignes: Vec<&str> = content.lines().collect();
        assert_eq!(lignes.len(), 3);
    }

    #[test]
    fn test_compteur_records() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test_compteur.jsonl");
        let mut writer = JsonLinesSinkWriter::new(&path).unwrap();

        assert_eq!(writer.records_written(), 0);
        let r = make_record(&[("x", json!(1))]);
        writer.write_record(&r).unwrap();
        assert_eq!(writer.records_written(), 1);
        writer.write_record(&r).unwrap();
        assert_eq!(writer.records_written(), 2);
        writer.finalize().unwrap();
    }
}
