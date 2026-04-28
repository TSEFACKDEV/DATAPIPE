// src/pipeline.rs
// 🎯 ORCHESTRATEUR PRINCIPAL - TSEFACK CALVIN KLEIN
// Ce module coordonne l'ensemble du pipeline ETL

use crate::config::{PipelineConfig, SourceConfig};
use crate::stats::ExecutionStats;
use crate::reader::SourceReader;
use crate::reader::csv_reader::CsvReader;
use crate::reader::json_reader::JsonReader;
use crate::reader::delimited_reader::DelimitedReader;
use crate::transform::Transform;
use crate::transform::factory::create_transform;
use crate::writer::factory::create_writer;
use anyhow::{Result, anyhow};
use std::path::Path;

/// Fonction principale du pipeline
/// 1. Charge la configuration TOML
/// 2. Crée les composants (lecteur, transformations, écrivain)
/// 3. Traite chaque record du début à la fin
/// 4. Affiche un rapport
pub fn run(config_path: &Path) -> Result<()> {
    println!("🔄 Initialisation du pipeline...");
    
    // 📋 ÉTAPE 1: Charger la configuration TOML
    let config = PipelineConfig::from_file(config_path)?;
    let mut stats = ExecutionStats::new();
    
    println!("📊 Configuration chargée:");
    println!("  Source: {} ({})", config.source.format, config.source.path);
    println!("  Destination: {} ({})", config.destination.format, config.destination.path);
    println!("  Transformations: {}", config.transforms.len());
    
    // 📖 ÉTAPE 2: Créer le lecteur selon le format source
    let reader = create_reader(&config.source)?;
    
    // ⚙️ ÉTAPE 3: Créer les transformations à partir de la config
    let transforms: Vec<Box<dyn Transform>> = config
        .transforms
        .iter()
        .map(|t| create_transform(t))
        .collect();
    
    println!("  Transformations créées: {} chaînes", transforms.len());
    
    // ✍️ ÉTAPE 4: Créer l'écrivain pour la sortie
    let mut writer = create_writer(&config.destination)?;
    
    println!("\n🚀 Traitement des records...");
    
    // 🔄 ÉTAPE 5: Boucle principale - traiter chaque record
    for result in reader.records() {
        match result {
            Ok(record) => {
                stats.records_read += 1;
                
                // Envelopper le record dans un Option pour le pipeline
                let mut record_option: Option<_> = Some(record);
                
                // Appliquer chaque transformation en chaîne
                for transform in &transforms {
                    record_option = match record_option {
                        Some(rec) => {
                            match transform.apply(rec) {
                                Some(new_rec) => {
                                    stats.records_transformed += 1;
                                    Some(new_rec)
                                }
                                None => {
                                    // Record a été filtré
                                    stats.records_filtered += 1;
                                    None
                                }
                            }
                        }
                        None => None, // Déjà filtré
                    };
                    
                    // Si le record est filtré, pas besoin de continuer les transformations
                    if record_option.is_none() {
                        break;
                    }
                }
                
                // Écrire le record si pas filtré
                if let Some(record) = record_option {
                    match writer.write_record(&record) {
                        Ok(_) => stats.records_written += 1,
                        Err(e) => {
                            stats.errors_encountered += 1;
                            eprintln!("⚠️ Erreur lors de l'écriture: {}", e);
                        }
                    }
                }
                
                // Afficher la progression tous les 1000 records
                if stats.records_read % 1000 == 0 {
                    println!("  → {} records traités...", stats.records_read);
                }
            }
            Err(e) => {
                stats.errors_encountered += 1;
                eprintln!("⚠️ Erreur lors de la lecture: {}", e);
            }
        }
    }
    
    // 📝 ÉTAPE 6: Finaliser l'écriture (flush des buffers)
    writer.finalize()?;
    
    // 📊 ÉTAPE 7: Afficher le rapport d'exécution
    stats.stop();
    println!("\n✅ Pipeline terminé!");
    stats.print_report();
    
    Ok(())
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// 🏭 FACTORIES - Fonctions pour créer les composants
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Factory pour créer le bon lecteur selon le format
/// 
/// # Arguments
/// * `config` - SourceConfig contenant format et chemin
/// 
/// # Returns
/// Un Box<dyn SourceReader> créé selon le format
/// 
/// # Formats supportés
/// - "csv" → CsvReader
/// - "json" → JsonReader
/// - "delimited" → DelimitedReader (tabulation, point-virgule, etc.)
fn create_reader(config: &SourceConfig) -> Result<Box<dyn SourceReader>> {
    match config.format.to_lowercase().as_str() {
        "csv" => {
            let delimiter = config
                .delimiter
                .as_ref()
                .and_then(|d| d.chars().next())
                .unwrap_or(',');
            
            Ok(Box::new(CsvReader {
                path: config.path.clone(),
                delimiter,
            }))
        }
        
        "json" => {
            Ok(Box::new(JsonReader {
                path: config.path.clone(),
            }))
        }
        
        "delimited" => {
            let delimiter = config
                .delimiter
                .as_ref()
                .and_then(|d| d.chars().next())
                .map(|c| c as u8)
                .ok_or_else(|| anyhow!("Délimiteur requis pour format 'delimited'"))?;
            
            Ok(Box::new(DelimitedReader {
                path: config.path.clone(),
                delimiter,
            }))
        }
        
        _ => Err(anyhow!(
            "Format source non supporté: '{}'. Utilisez: csv, json, delimited",
            config.format
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_initialization() {
        // TODO: Ajouter tests d'intégration (TSEFACK #01)
        // - Test avec fichier TOML valide
        // - Test avec fichier TOML invalide
        // - Test avec fichiers sources manquants
    }

    #[test]
    fn test_reader_factory() {
        // TODO: Tester la création de lecteurs
    }
}