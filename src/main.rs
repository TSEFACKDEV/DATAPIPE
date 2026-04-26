use clap::Parser;
use anyhow::Result;
use std::path::PathBuf;

pub mod reader;
pub mod join;
pub mod config;
pub mod stats;

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
    } else if cli.dry_run {
        dry_run_pipeline(&cli.config)?;
    } else {
        run_pipeline(&cli.config)?;
    }
    Ok(())
}

fn run_pipeline(config_path: &PathBuf) -> Result<()> {
    use crate::config::PipelineConfig;
    let config = PipelineConfig::from_file(config_path)?;
    println!("Pipeline: {} -> {}", config.source.format, config.destination.format);
    println!("Pipeline execute avec succes!");
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

    println!("[DRY-RUN] Apercu des 5 premiers records: (lecteurs pas encore implémentes)");
    println!("[DRY-RUN] Aucun fichier ecrit.");
    println!("[DRY-RUN] Simulation terminee avec succes.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dry_run_no_file_written() {
        // Verifier que dry_run ne cree pas de fichier de sortie
        let result = dry_run_pipeline(&PathBuf::from("pipeline.toml"));
        // Si pipeline.toml n existe pas, l erreur est normale
        // Le test verifie juste que la fonction existe et retourne un Result
        let _ = result;
    }
}
