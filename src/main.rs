use clap::Parser;
use anyhow::Result;
use std::path::PathBuf;

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
        // watch_mode(&cli.config, cli.interval)?;
    } else if cli.dry_run {
        // TODO: Implémenter le mode dry-run (ATEKOUMBO #09)
        println!("🔍 Mode dry-run activé");
        // dry_run_pipeline(&cli.config)?;
    } else {
        // Mode normal
        run_pipeline(&cli.config)?;
    }
    
    Ok(())
}

fn run_pipeline(config_path: &PathBuf) -> Result<()> {
    // TODO: Implémenter l'orchestrateur principal (TSEFACK #01)
    // 1. Charger la configuration TOML
    // 2. Créer le lecteur approprié
    // 3. Appliquer les transformations
    // 4. Écrire les résultats
    // 5. Afficher les statistiques
    
    println!("✅ Pipeline exécuté avec succès!");
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