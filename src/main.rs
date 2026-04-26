use clap::Parser;
use anyhow::Result;
use std::path::PathBuf;

// Import du module pipeline pour accéder à run_pipeline
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
    /// Chemin vers le fichier de configuration TOML
    #[arg(short, long, default_value = "pipeline.toml")]
    config: PathBuf,

    /// Mode dry-run (simulation sans écriture)
    #[arg(long)]
    dry_run: bool,

    /// Mode watch (surveillance du fichier source)
    #[arg(long)]
    watch: bool,

    /// Intervalle de surveillance en secondes (pour --watch)
    #[arg(long, default_value = "30")]
    interval: u64,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    println!("🚀 DataPipe - Démarrage...");
    println!("📁 Configuration: {:?}", cli.config);
    
    if cli.watch {
        // TODO: Implémenter le mode watch (NJOH #10)
        println!("👀 Mode watch activé (intervalle: {}s)", cli.interval);
        // watch::watch_pipeline(&cli.config, cli.interval)?;
    } else if cli.dry_run {
        // TODO: Implémenter le mode dry-run (ATEKOUMBO #09)
        println!("🔍 Mode dry-run activé");
        // dry_run_pipeline(&cli.config)?;
    } else {
        // Mode normal - Lance le pipeline principal
        pipeline::run(&cli.config)?;
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parsing() {
        // TODO: Ajouter des tests CLI
    }
}