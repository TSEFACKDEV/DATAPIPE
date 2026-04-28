use clap::Parser;
use anyhow::Result;
use std::path::PathBuf;

mod config;
mod reader;
mod transform;
mod writer;
mod pipeline;
mod stats;
mod validation;
mod report;
mod join;
mod watch;

#[derive(Parser, Debug)]
#[command(name = "datapipe")]
#[command(about = "DataPipe - Outil ETL en Rust", long_about = None)]
struct Cli {
    #[arg(short, long, default_value = "pipeline.toml")]
    config: PathBuf,

    #[arg(long)]
    dry_run: bool,

    #[arg(long)]
    watch: bool,

    #[arg(long, default_value = "30")]
    interval: u64,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    println!("DataPipe - Demarrage...");
    println!("Configuration: {:?}", cli.config);

    if cli.watch {
        println!("Mode watch active (intervalle: {}s)", cli.interval);
        watch::watch_mode(&cli.config, cli.interval)?;
    } else if cli.dry_run {
        dry_run_pipeline(&cli.config)?;
    } else {
        pipeline::run(&cli.config)?;
    }
    Ok(())
}

fn dry_run_pipeline(config_path: &PathBuf) -> Result<()> {
    use crate::config::PipelineConfig;

    println!("[DRY-RUN] Chargement de la configuration...");
    let config = PipelineConfig::from_file(config_path)?;

    println!("[DRY-RUN] Source     : {} ({})", config.source.path, config.source.format);
    println!("[DRY-RUN] Destination: {} ({})", config.destination.path, config.destination.format);
    println!("[DRY-RUN] Transformations: {}", config.transforms.len());

    for (i, t) in config.transforms.iter().enumerate() {
        println!("  [{}] type={}", i + 1, t.r#type);
    }

    if let Some(join) = &config.join {
        println!("[DRY-RUN] JOIN: {} sur '{}' = '{}'",
            join.join_type, join.left_key, join.right_key);
    }

    // Aperçu des 5 premiers records depuis la source
    match pipeline::create_reader(&config.source) {
        Ok(reader) => {
            let preview: Vec<_> = reader.records()
                .take(5)
                .filter_map(|r| r.ok())
                .collect();
            if preview.is_empty() {
                println!("[DRY-RUN] Aucun record trouvé dans la source.");
            } else {
                println!("[DRY-RUN] Aperçu des {} premier(s) record(s):", preview.len());
                for (i, rec) in preview.iter().enumerate() {
                    let fields: Vec<String> = rec.iter()
                        .map(|(k, v)| format!("{}: {}", k, v))
                        .collect();
                    println!("  [{}] {{ {} }}", i + 1, fields.join(", "));
                }
            }
        }
        Err(e) => println!("[DRY-RUN] Impossible de lire la source: {}", e),
    }

    println!("[DRY-RUN] Aucun fichier écrit.");
    println!("[DRY-RUN] Simulation terminée avec succès.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dry_run_with_valid_config() {
        // pipeline.toml doit exister à la racine du projet
        let result = dry_run_pipeline(&PathBuf::from("pipeline.toml"));
        assert!(result.is_ok(), "dry_run doit réussir avec pipeline.toml valide");
    }

    #[test]
    fn test_dry_run_with_missing_config() {
        let result = dry_run_pipeline(&PathBuf::from("nonexistent_config.toml"));
        assert!(result.is_err(), "dry_run doit échouer si le fichier config est absent");
    }
}
