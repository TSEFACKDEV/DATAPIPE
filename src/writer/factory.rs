// src/writer/factory.rs
//
// Auteur : NGANSOP NGOUABOU FREDI LOIK (#07)
// Rôle   : Fabrique d'écrivains (Factory Pattern)
//
// ─── CONCEPT : FACTORY PATTERN ───────────────────────────────────────────────
//
// Le Factory Pattern (Patron de conception Fabrique) est l'un des design
// patterns les plus fondamentaux en génie logiciel.
//
// PROBLÈME sans Factory :
//   L'orchestrateur (pipeline.rs) devrait connaître chaque type d'écrivain :
//
//   ```rust
//   // MAUVAISE approche → dépendance directe, difficile à étendre
//   let writer: Box<dyn SinkWriter> = if config.format == "csv" {
//       Box::new(CsvSinkWriter::new(...)?)
//   } else if config.format == "json" {
//       Box::new(JsonSinkWriter::new(...)?)
//   } else if config.format == "jsonl" {
//       Box::new(JsonLinesSinkWriter::new(...)?)
//   } else {
//       return Err(anyhow!("Format inconnu"))
//   };
//   ```
//
// SOLUTION avec Factory :
//   Ce détail est encapsulé dans `create_writer()`.
//   L'orchestrateur appelle juste `create_writer(&config)` sans se soucier
//   de quelle classe concrète est instanciée.
//
// ─── AVANTAGES ────────────────────────────────────────────────────────────────
// 1. Open/Closed : ajouter "parquet" → seulement une branche dans create_writer
// 2. Single Responsibility : l'orchestrateur orchestre, la factory instancie
// 3. Testabilité : on peut tester la factory indépendamment
//
// ─── LIEN AVEC LE FICHIER TOML ────────────────────────────────────────────────
// La configuration pipeline.toml contient :
//   [destination]
//   format = "jsonl"
//   path = "output/resultat.jsonl"
//
// create_writer() lit config.format pour décider quelle struct créer.
// ────────────────────────────────────────────────────────────────────────────

use super::SinkWriter;
use super::csv_writer::CsvSinkWriter;
use super::json_writer::JsonSinkWriter;
use super::jsonl_writer::JsonLinesSinkWriter;
use crate::config::DestinationConfig;
use anyhow::{anyhow, Result};

/// Crée l'écrivain approprié à partir de la configuration de destination.
///
/// # Arguments
/// * `config` - Configuration destination lue depuis le fichier TOML
///
/// # Formats supportés
/// | config.format | Type créé                | Comportement                          |
/// |---------------|--------------------------|---------------------------------------|
/// | `"csv"`       | `CsvSinkWriter`          | Fichier CSV avec en-têtes             |
/// | `"json"`      | `JsonSinkWriter`         | Tableau JSON (accumulé en mémoire)    |
/// | `"jsonl"`     | `JsonLinesSinkWriter`    | JSONL streaming (mémoire constante)   |
///
/// # Erreurs
/// - Format non reconnu → `Err` avec message explicite
/// - Fichier non créable → `Err` propagé depuis le constructeur de l'écrivain
///
/// # Exemple d'utilisation (dans pipeline.rs)
/// ```rust,ignore
/// let config = DestinationConfig { format: "jsonl".to_string(), path: "out.jsonl".to_string() };
/// let mut writer = create_writer(&config)?;
/// writer.write_record(&mon_record)?;
/// writer.finalize()?;
/// ```
pub fn create_writer(config: &DestinationConfig) -> Result<Box<dyn SinkWriter>> {
    match config.format.as_str() {
        "csv" => {
            // CsvSinkWriter a besoin des en-têtes au moment de la construction.
            // Ici on passe None → les en-têtes seront inférées du premier record.
            // Le writer CSV (NGLITANG #06) gère ce cas avec headers_written: false.
            let writer = CsvSinkWriter::new(&config.path, None)?;
            Ok(Box::new(writer))
        }

        "json" => {
            // JsonSinkWriter accumule tous les records en mémoire puis écrit
            // un tableau JSON complet lors de finalize().
            // ⚠️  Attention : non adapté aux très gros volumes (100k+ records).
            // Pour les gros volumes, préférer "jsonl".
            let writer = JsonSinkWriter::new(&config.path);
            Ok(Box::new(writer))
        }

        "jsonl" => {
            // JsonLinesSinkWriter écrit chaque record immédiatement (streaming).
            // Recommandé pour les gros volumes : mémoire O(1).
            let writer = JsonLinesSinkWriter::new(&config.path)?;
            Ok(Box::new(writer))
        }

        format_inconnu => {
            // Erreur claire avec les formats supportés → aide l'utilisateur
            // à corriger son fichier pipeline.toml immédiatement.
            Err(anyhow!(
                "Format de destination '{format_inconnu}' non reconnu. \
                 Formats supportés : csv, json, jsonl.\n\
                 Vérifiez votre pipeline.toml :\n\
                 [destination]\n\
                 format = \"jsonl\"  # ← doit être l'un des formats ci-dessus\n\
                 path = \"output/mon_fichier.jsonl\""
            ))
        }
    }
}

// ─── TESTS UNITAIRES ─────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::tempdir;

    fn make_config(format: &str, path: &str) -> DestinationConfig {
        DestinationConfig {
            format: format.to_string(),
            path: path.to_string(),
        }
    }

    // ─── TEST 1 : Factory crée un écrivain JSONL fonctionnel ─────────────
    #[test]
    fn test_factory_cree_jsonl() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.jsonl").to_str().unwrap().to_string();

        let config = make_config("jsonl", &path);
        let mut writer = create_writer(&config)
            .expect("La factory doit créer un writer JSONL sans erreur");

        let record: super::super::Record = [
            ("nom".to_string(), json!("TestJSONL")),
        ].into_iter().collect();

        writer.write_record(&record).unwrap();
        writer.finalize().unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(!content.is_empty(), "Le fichier JSONL ne doit pas être vide");
        assert!(content.contains("TestJSONL"));
    }

    // ─── TEST 2 : Factory crée un écrivain JSON fonctionnel ──────────────
    #[test]
    fn test_factory_cree_json() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.json").to_str().unwrap().to_string();

        let config = make_config("json", &path);
        let mut writer = create_writer(&config)
            .expect("La factory doit créer un writer JSON sans erreur");

        let record: super::super::Record = [
            ("nom".to_string(), json!("TestJSON")),
        ].into_iter().collect();

        writer.write_record(&record).unwrap();
        writer.finalize().unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("TestJSON"));
    }

    // ─── TEST 3 : Format inconnu → erreur avec message utile ─────────────
    #[test]
    fn test_format_inconnu_retourne_erreur() {
        let config = make_config("parquet", "/tmp/test.parquet");
        match create_writer(&config) {
            Err(e) => {
                let msg = format!("{}", e);
                assert!(msg.contains("parquet"),
                    "L'erreur doit mentionner 'parquet' : {}", msg);
                assert!(msg.contains("csv") || msg.contains("json"),
                    "L'erreur doit lister les formats supportés : {}", msg);
            }
            Ok(_) => panic!("Un format inconnu doit retourner une erreur"),
        }
    }

    // ─── TEST 4 : Format vide → erreur ───────────────────────────────────
    #[test]
    fn test_format_vide_retourne_erreur() {
        let config = make_config("", "/tmp/test.txt");
        assert!(create_writer(&config).is_err());
    }

    // ─── TEST 5 : JSONL produit du format valide ──────────────────────────
    #[test]
    fn test_jsonl_format_valide() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("valid.jsonl").to_str().unwrap().to_string();

        let config = make_config("jsonl", &path);
        let mut writer = create_writer(&config).unwrap();

        for i in 0..5 {
            let record: super::super::Record = [
                ("id".to_string(), json!(i)),
                ("val".to_string(), json!(format!("item_{}", i))),
            ].into_iter().collect();
            writer.write_record(&record).unwrap();
        }
        writer.finalize().unwrap();

        // Chaque ligne doit être du JSON valide
        let content = std::fs::read_to_string(&path).unwrap();
        for (i, ligne) in content.lines().enumerate() {
            serde_json::from_str::<serde_json::Value>(ligne)
                .unwrap_or_else(|_| panic!("Ligne {} n'est pas du JSON valide: {}", i+1, ligne));
        }
        assert_eq!(content.lines().count(), 5);
    }
}
