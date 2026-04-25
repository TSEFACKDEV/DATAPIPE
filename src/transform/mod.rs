// src/transform/mod.rs
pub mod rename;
pub mod filter;
pub mod cast;
pub mod compute;
pub mod drop;
pub mod factory;

use super::reader::Record;

/// Trait Transform : contrat pour toutes les transformations
pub trait Transform: Send + Sync {
    /// Applique la transformation sur un Record
    /// Retourne None si le record doit être filtré
    fn apply(&self, record: Record) -> Option<Record>;
    
    /// Nom de la transformation pour les logs
    fn name(&self) -> &str;
}