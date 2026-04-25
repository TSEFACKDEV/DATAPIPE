pub mod csv_reader;
pub mod json_reader;
pub mod delimited_reader;

use std::collections::HashMap;
use serde_json::Value;

/// Type Record : une ligne de données sous forme de HashMap
pub type Record = HashMap<String, Value>;

/// Trait SourceReader : contrat pour tous les lecteurs
pub trait SourceReader {
    /// Retourne un itérateur sur les Records
    fn records(&self) -> Box<dyn Iterator<Item = anyhow::Result<Record>>>;
}