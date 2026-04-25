use super::{Transform, rename::RenameTransform, filter::FilterTransform, 
            cast::CastTransform, compute::ComputeTransform, drop::DropTransform};
use crate::config::TransformConfig;

/// Crée une transformation à partir de sa configuration
pub fn create_transform(config: &TransformConfig) -> Box<dyn Transform> {
    // TODO: Implémenter la fabrique (NOLACK #05)
    // Match sur config.type pour créer la bonne transformation
    todo!("Implémenter la fabrique de transformations")
}