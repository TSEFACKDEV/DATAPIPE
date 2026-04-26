// src/lib.rs
// Expose les modules du projet pour les tests d'intégration (dossier tests/).
// Les tests d'intégration importent via `use datapipe::...` — ils ont besoin
// que les modules soient publiquement accessibles depuis la lib.

pub mod reader;
pub mod config;
pub mod pipeline;
pub mod stats;
pub mod validation;
pub mod report;
pub mod join;
pub mod watch;

// Réexporter les types les plus utilisés pour simplifier les imports
pub use reader::{SourceReader, Record};
pub use reader::csv_reader::CsvReader;