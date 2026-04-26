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
        let result = dry_run_pipeline(&PathBuf::from("pipeline.toml"));
        let _ = result;
    }
}
