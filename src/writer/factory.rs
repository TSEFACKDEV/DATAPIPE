// src/writer/factory.rs
use super::SinkWriter;
use crate::config::DestinationConfig;

/// Crée un écrivain à partir de la configuration
pub fn create_writer(config: &DestinationConfig) -> Box<dyn SinkWriter> {
    // TODO: Implémenter la fabrique d'écrivains (NGANSOP #07)
    todo!("Implémenter la fabrique d'écrivains")
}