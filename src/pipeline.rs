// src/pipeline.rs
//  ORCHESTRATEUR PRINCIPAL - TSEFACK CALVIN KLEIN
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
use crate::validation::validate_record;
use crate::report::generate_html_report;
use anyhow::{Result, anyhow};
use std::path::Path;

/// Fonction principale du pipeline
/// 1. Charge la configuration TOML
/// 2. Crée les composants (lecteur, transformations, écrivain)
/// 3. Traite chaque record du début à la fin
/// 4. Affiche un rapport
pub fn run(config_path: &Path) -> Result<()> {
    println!(" Initialisation du pipeline...");
    
    //  ÉTAPE 1: Charger la configuration TOML
    let config = PipelineConfig::from_file(config_path)?;
    let mut stats = ExecutionStats::new();
    
    // Renseigner les infos de source/destination dans les stats
    stats.source_path = config.source.path.clone();
    stats.source_format = config.source.format.clone();
    stats.destination_path = config.destination.path.clone();
    stats.destination_format = config.destination.format.clone();
    stats.transforms_count = config.transforms.len();
    
    println!(" Configuration chargée:");
    println!("  Source: {} ({})", config.source.format, config.source.path);
    println!("  Destination: {} ({})", config.destination.format, config.destination.path);
    println!("  Transformations: {}", config.transforms.len());
    
    if let Some(ref schema) = config.schema {
        println!("  Validation: {} colonnes requises", schema.required_columns.len());
    }
    
    //  ÉTAPE 2: Créer le lecteur selon le format source
    let reader = create_reader(&config.source)?;
    
    //  ÉTAPE 3: Créer les transformations à partir de la config
    let transforms: Vec<Box<dyn Transform>> = config
        .transforms
        .iter()
        .map(|t| create_transform(t))
        .collect();
    
    println!("  Transformations créées: {} chaînes", transforms.len());
    
    //  ÉTAPE 4: Créer l'écrivain pour la sortie
    let mut writer = create_writer(&config.destination)?;
    
    println!("\n Traitement des records...");

    // Aperçu des premiers records pour le rapport HTML
    let mut preview: Vec<crate::reader::Record> = Vec::new();

    //  ÉTAPE 5: Boucle principale - traiter chaque record
    for result in reader.records() {
        match result {
            Ok(record) => {
                stats.records_read += 1;

                //  Validation de schéma (si configurée)
                if let Some(ref schema) = config.schema {
                    let errors = validate_record(&record, schema);
                    if !errors.is_empty() {
                        stats.errors_encountered += 1;
                        eprintln!("[WARN] Validation échouée pour le record #{}: {} erreur(s)",
                            stats.records_read, errors.len());
                        for err in &errors {
                            eprintln!("   - {}", err.to_string());
                        }
                    }
                }

                // Collecter les 10 premiers records pour l'aperçu du rapport
                if preview.len() < 10 {
                    preview.push(record.clone());
                }

                //  Mise à jour des stats par colonne (valeurs numériques)
                for (col, val) in &record {
                    if let Some(n) = val.as_f64() {
                        stats.update_column_numeric(col, n);
                    } else if val.is_null() || val.as_str().map(|s| s.is_empty()).unwrap_or(false) {
                        stats.record_column_null(col);
                    }
                }
                
                // Envelopper le record dans un Option pour le pipeline
                let mut record_option: Option<_> = Some(record);
                let had_transforms = !transforms.is_empty();
                
                // Appliquer chaque transformation en chaîne
                for transform in &transforms {
                    record_option = match record_option {
                        Some(rec) => {
                            match transform.apply(rec) {
                                Some(new_rec) => Some(new_rec),
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
                
                // Compter une seule fois les records transformés (non filtrés)
                if had_transforms && record_option.is_some() {
                    stats.records_transformed += 1;
                }
                
                // Écrire le record si pas filtré
                if let Some(record) = record_option {
                    match writer.write_record(&record) {
                        Ok(_) => stats.records_written += 1,
                        Err(e) => {
                            stats.errors_encountered += 1;
                            eprintln!("[WARN] Erreur lors de l'écriture: {}", e);
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
                eprintln!("[WARN] Erreur lors de la lecture: {}", e);
            }
        }
    }
    
    //  ÉTAPE 6: Finaliser l'écriture (flush des buffers)
    writer.finalize()?;
    
    //  ÉTAPE 7: Afficher le rapport d'exécution
    stats.stop();
    println!("\n[OK] Pipeline terminé!");
    stats.print_report();

    //  ÉTAPE 8: Générer le rapport HTML
    let report_path = "reports/rapport.html";
    match generate_html_report(&stats, &preview, report_path) {
        Ok(_) => println!(" Rapport HTML disponible : {}", report_path),
        Err(e) => eprintln!("[WARN] Impossible de générer le rapport HTML : {}", e),
    }

    Ok(())
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  FACTORIES - Fonctions pour créer les composants
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Factory pour créer le bon lecteur selon le format
/// Exposée publiquement pour permettre le dry-run et les tests.
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
pub fn create_reader(config: &SourceConfig) -> Result<Box<dyn SourceReader>> {
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
    use crate::config::SourceConfig;

    #[test]
    fn test_reader_factory_csv() {
        let config = SourceConfig {
            format: "csv".to_string(),
            path: "data/test.csv".to_string(),
            delimiter: Some(",".to_string()),
        };
        assert!(create_reader(&config).is_ok());
    }

    #[test]
    fn test_reader_factory_json() {
        let config = SourceConfig {
            format: "json".to_string(),
            path: "data/test.json".to_string(),
            delimiter: None,
        };
        assert!(create_reader(&config).is_ok());
    }

    #[test]
    fn test_reader_factory_delimited_missing_delimiter() {
        let config = SourceConfig {
            format: "delimited".to_string(),
            path: "data/villes.txt".to_string(),
            delimiter: None, // manquant → doit retourner Err
        };
        assert!(create_reader(&config).is_err());
    }

    #[test]
    fn test_reader_factory_unknown_format() {
        let config = SourceConfig {
            format: "parquet".to_string(),
            path: "data/file.parquet".to_string(),
            delimiter: None,
        };
        assert!(create_reader(&config).is_err());
    }
}