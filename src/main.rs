//! # DataPipe — Point d'entrée CLI
//!
//! Ce module définit l'interface en ligne de commande (CLI) et dispatche
//! vers les différents modes d'exécution :
//!
//! - **Mode normal** : exécution unique du pipeline.
//! - **Mode `--dry-run`** : simulation sans écriture de fichier de sortie.
//! - **Mode `--watch`** : surveillance continue du fichier source.
//!
//! ## Exemples d'utilisation
//!
//! ```bash
//! # Mode normal
//! datapipe --config pipeline.toml
//!
//! # Mode dry-run
//! datapipe --config pipeline.toml --dry-run
//!
//! # Mode watch (vérification toutes les 30 secondes)
//! datapipe --config pipeline.toml --watch --interval 30
//! ```

mod config;
mod pipeline;
mod reader;
mod transform;
mod writer;
mod stats;
mod validation;
mod report;
mod join;
pub mod watch;

use clap::Parser;
use anyhow::Result;
use std::path::PathBuf;

/// Arguments de la ligne de commande DataPipe.
#[derive(Parser, Debug)]
#[command(name = "datapipe")]
#[command(about = "DataPipe - Outil ETL en Rust | Groupe 6 ENSP Yaoundé")]
#[command(version = "1.0.0")]
struct Cli {
    /// Chemin vers le fichier de configuration TOML
    #[arg(short, long, default_value = "pipeline.toml")]
    config: PathBuf,

    /// Mode dry-run : simule le pipeline sans écrire le fichier de sortie
    #[arg(long)]
    dry_run: bool,

    /// Mode watch : surveille le fichier source et relance le pipeline à chaque modification
    #[arg(long)]
    watch: bool,

    /// Intervalle de surveillance en secondes (utilisé avec --watch)
    #[arg(long, default_value = "30")]
    interval: u64,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    println!("🚀 DataPipe - Démarrage...");
    println!("📁 Configuration: {:?}", cli.config);

    if cli.watch {
        // Mode watch — implémenté par NJOH #10
        watch::watch_mode(&cli.config, cli.interval)?;
    } else if cli.dry_run {
        // Mode dry-run — implémenté par ATEKOUMBO #09
        println!("🔍 Mode dry-run activé");
        // dry_run_pipeline(&cli.config)?;
        println!("⚠️  Mode dry-run non encore intégré (ATEKOUMBO #09).");
    } else {
        // Mode normal
        pipeline::run(&cli.config)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_default_config() {
        // Vérifie que le parsing fonctionne avec les valeurs par défaut
        let cli = Cli::try_parse_from(["datapipe"]).unwrap();
        assert_eq!(cli.config, PathBuf::from("pipeline.toml"));
        assert!(!cli.dry_run);
        assert!(!cli.watch);
        assert_eq!(cli.interval, 30);
    }

    #[test]
    fn test_cli_watch_flag() {
        let cli = Cli::try_parse_from(["datapipe", "--watch", "--interval", "10"]).unwrap();
        assert!(cli.watch);
        assert_eq!(cli.interval, 10);
    }

    #[test]
    fn test_cli_dry_run_flag() {
        let cli = Cli::try_parse_from(["datapipe", "--dry-run"]).unwrap();
        assert!(cli.dry_run);
    }
}
