// src/pipeline.rs
use crate::config::PipelineConfig;
use crate::stats::ExecutionStats;
use anyhow::Result;

pub fn run(config_path: &std::path::Path) -> Result<()> {
    // Charger la configuration
    let config = PipelineConfig::from_file(config_path)?;
    let mut stats = ExecutionStats::new();
    
    println!("📊 Configuration chargée: {} -> {}", config.source.format, config.destination.format);
    
    // TODO: Implémenter l'orchestrateur (TSEFACK #01)
    // 1. Créer le lecteur selon config.source.format
    // 2. Créer les transformations selon config.transforms
    // 3. Créer l'écrivain selon config.destination.format
    // 4. Pour chaque record du lecteur:
    //    - Appliquer les transformations
    //    - Écrire le résultat
    //    - Mettre à jour les statistiques
    // 5. Finaliser l'écriture
    // 6. Afficher le rapport
    
    stats.stop();
    stats.print_report();
    
    Ok(())
}