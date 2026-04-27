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
//
// ─── PATTERN TECHNIQUE : BufWriter ──────────────────────────────────────────
// BufWriter accumule les écritures en RAM (tampon 8Ko) et vide en un appel.
// Sans BufWriter : 1 appel système par ligne → lent.
// Avec BufWriter : 1 appel système tous les 8Ko → 10-100x plus rapide.
// ────────────────────────────────────────────────────────────────────────────

use super::{Record, SinkWriter};
use std::io::Write;

pub struct JsonLinesSinkWriter<W: Write> {
    writer: W,
}

impl<W: Write> SinkWriter for JsonLinesSinkWriter<W> {
    fn write_record(&mut self, record: &Record) -> anyhow::Result<()> {
        // TODO: Implémenter l'écriture JSONL (NGANSOP #07)
        // 1. Sérialiser le record en JSON
        // 2. Écrire une ligne
        todo!("Implémenter l'écrivain JSONL")
    }

    fn finalize(&mut self) -> anyhow::Result<()> {
        // TODO: Flush final
        
        todo!("Implémenter finalize JSONL")
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
    ///
    /// Crée automatiquement les répertoires parents si nécessaire.
    ///
    /// # Erreurs
    /// - Répertoire parent non créable (permissions)
    /// - Fichier non créable
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
    pub fn records_written(&self) -> usize {
        self.records_written
    }
}

impl SinkWriter for JsonLinesSinkWriter {
    /// Sérialise le record en JSON compact sur une ligne unique.
    ///
    /// # Algorithme
    /// 1. `serde_json::to_string(record)` → JSON compact (sans indentation)
    ///    → OBLIGATOIRE : un pretty-print ajouterait des `\n` internes
    ///      qui invalideraient le format JSONL (1 objet = 1 ligne).
    /// 2. `writeln!` → écrit la chaîne + '\n' dans le buffer
    fn write_record(&mut self, record: &Record) -> Result<()> {
        let json_line = serde_json::to_string(record).with_context(|| {
            format!("Erreur de sérialisation JSON pour le record #{}", self.records_written + 1)
        })?;

        writeln!(self.writer, "{}", json_line).with_context(|| {
            format!("Erreur d'écriture JSONL pour le record #{}", self.records_written + 1)
        })?;

        self.records_written += 1;
        Ok(())
    }

    /// Vide le buffer et garantit l'écriture physique sur le disque.
    ///
    /// # CRITIQUE : Sans flush(), les dernières données sont perdues !
    ///
    /// BufWriter garde les données en RAM jusqu'à ce que son buffer soit plein
    /// (8192 octets). Si le programme termine sans flush(), les données du
    /// buffer partiel ne sont JAMAIS écrites sur disque → FICHIER TRONQUÉ.
    ///
    /// Exemple : 100 records × 50 octets = 5000 octets < 8192 (taille buffer)
    /// → Sans finalize() : 0 octet écrit sur le disque !
    fn finalize(&mut self) -> Result<()> {
        self.writer
            .flush()
            .context("Erreur lors du flush du buffer JSONL")
    }
}

// ─── TESTS UNITAIRES ─────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};
    use std::collections::HashMap;
    use tempfile::tempdir;

    /// Crée un Record de test à partir de paires clé-valeur
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

        let parsed: Value = serde_json::from_str(lignes[0]).unwrap();
        assert_eq!(parsed["nom"], json!("Jean"));
        assert_eq!(parsed["age"], json!(25));
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

        for (i, ligne) in lignes.iter().enumerate() {
            let parsed: Value = serde_json::from_str(ligne).unwrap();
            assert_eq!(parsed["nom"], json!(noms[i]));
        }
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

    #[test]
    fn test_types_mixtes() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test_types.jsonl");
        let mut writer = JsonLinesSinkWriter::new(&path).unwrap();

        let record = make_record(&[
            ("texte", json!("Douala")),
            ("entier", json!(42)),
            ("flottant", json!(3.14)),
            ("booleen", json!(true)),
            ("vide", Value::Null),
        ]);
        writer.write_record(&record).unwrap();
        writer.finalize().unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        let parsed: Value = serde_json::from_str(content.lines().next().unwrap()).unwrap();
        assert_eq!(parsed["texte"], json!("Douala"));
        assert_eq!(parsed["entier"], json!(42));
        assert_eq!(parsed["booleen"], json!(true));
        assert_eq!(parsed["vide"], Value::Null);
    }

    #[test]
    fn test_format_compact_une_ligne_par_record() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test_compact.jsonl");
        let mut writer = JsonLinesSinkWriter::new(&path).unwrap();

        let record = make_record(&[("a", json!("v")), ("b", json!(99))]);
        writer.write_record(&record).unwrap();
        writer.finalize().unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        let lignes_non_vides: Vec<&str> = content.lines().filter(|l| !l.is_empty()).collect();
        assert_eq!(lignes_non_vides.len(), 1);
    }

    #[test]
    fn test_creation_repertoire_parent() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("sous_dossier/profond/resultats.jsonl");

        let mut writer = JsonLinesSinkWriter::new(&path)
            .expect("Doit créer les répertoires parents manquants");

        let record = make_record(&[("test", json!(true))]);
        writer.write_record(&record).unwrap();
        writer.finalize().unwrap();

        assert!(path.exists());
    }

    #[test]
    fn test_finalize_sans_record() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test_vide.jsonl");
        let mut writer = JsonLinesSinkWriter::new(&path).unwrap();
        writer.finalize().unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.is_empty());
    }

    #[test]
    fn test_mille_records() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test_1000.jsonl");
        let mut writer = JsonLinesSinkWriter::new(&path).unwrap();

        for i in 0..1000usize {
            let record = make_record(&[
                ("id", json!(i)),
                ("nom", json!(format!("Employe_{}", i))),
                ("salaire", json!(30000 + i * 100)),
            ]);
            writer.write_record(&record).unwrap();
        }
        writer.finalize().unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        assert_eq!(content.lines().count(), 1000);

        for (i, ligne) in content.lines().enumerate() {
            let parsed: Value = serde_json::from_str(ligne).unwrap();
            assert_eq!(parsed["id"], json!(i));
        }
    }

    #[test]
    fn test_record_vide() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test_vide_record.jsonl");
        let mut writer = JsonLinesSinkWriter::new(&path).unwrap();
        let record: Record = HashMap::new();
        writer.write_record(&record).unwrap();
        writer.finalize().unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        assert_eq!(content.lines().next().unwrap(), "{}");
    }
}
