//! # Module Watch — Surveillance en temps réel
//!
//! Ce module implémente le **mode watch** de DataPipe : il surveille le fichier source
//! défini dans la configuration TOML et relance automatiquement le pipeline dès qu'une
//! modification est détectée.
//!
//! ## Principe de fonctionnement
//!
//! Le mode watch utilise une boucle de **polling** : toutes les `interval_secs` secondes,
//! il compare la date de dernière modification (`mtime`) du fichier source avec la valeur
//! mémorisée. Si elle a changé, il relance [`crate::pipeline::run`].
//!
//! ## Exemple d'utilisation (CLI)
//!
//! ```bash
//! datapipe --watch --config pipeline.toml --interval 30
//! ```
//!
//! ## Limitations connues
//!
//! - Seul le fichier **source** est surveillé (pas le fichier de configuration).
//! - Le mode watch tourne indéfiniment ; interrompre avec `Ctrl+C`.

use std::path::Path;
use std::time::{Duration, SystemTime};

use anyhow::{Context, Result};

/// Retourne la date de dernière modification (`mtime`) d'un fichier.
///
/// # Erreurs
/// Retourne une erreur si le fichier n'existe pas ou si les métadonnées sont inaccessibles.
fn get_mtime(path: &Path) -> Result<SystemTime> {
    let metadata = std::fs::metadata(path)
        .with_context(|| format!("Impossible de lire les métadonnées de {:?}", path))?;
    metadata
        .modified()
        .context("Le système de fichiers ne supporte pas mtime")
}

/// Récupère le chemin du fichier source depuis le fichier de configuration TOML.
///
/// # Erreurs
/// Retourne une erreur si le TOML est illisible ou mal formé.
fn get_source_path(config_path: &Path) -> Result<String> {
    let content = std::fs::read_to_string(config_path)
        .with_context(|| format!("Impossible de lire {:?}", config_path))?;
    let config: crate::config::PipelineConfig =
        toml::from_str(&content).context("Fichier de configuration TOML invalide")?;
    Ok(config.source.path)
}

/// Lance le pipeline une fois et affiche le résultat.
///
/// Les erreurs d'exécution sont affichées mais n'interrompent pas le mode watch,
/// pour tolérer les fichiers source temporairement invalides.
fn run_pipeline_once(config_path: &Path) {
    println!("▶  Lancement du pipeline...");
    match crate::pipeline::run(config_path) {
        Ok(()) => println!("✅ Pipeline terminé avec succès."),
        Err(e) => eprintln!("❌ Erreur durant l'exécution : {:#}", e),
    }
}

/// Mode surveillance — relance le pipeline à chaque modification du fichier source.
///
/// Cette fonction **bloque indéfiniment** (Ctrl+C pour arrêter).
///
/// ## Algorithme
/// 1. Lit le chemin source depuis la configuration.
/// 2. Mémorise son `mtime` initial.
/// 3. Lance une première exécution immédiate.
/// 4. Boucle : attend `interval_secs` s, compare le `mtime`, relance si changement.
///
/// # Arguments
/// * `config_path` — Chemin vers `pipeline.toml`.
/// * `interval_secs` — Intervalle de vérification en secondes (ex : 30).
///
/// # Erreurs
/// Retourne une erreur uniquement si l'initialisation échoue (config introuvable, etc.).
///
/// # Exemple
///
/// ```no_run
/// use std::path::Path;
/// datapipe::watch::watch_mode(Path::new("pipeline.toml"), 30).unwrap();
/// ```
pub fn watch_mode(config_path: &Path, interval_secs: u64) -> Result<()> {
    let source_path_str = get_source_path(config_path)
        .context("Impossible de déterminer le fichier source depuis la configuration")?;
    let source_path = Path::new(&source_path_str);

    println!("👀 Mode watch activé");
    println!("   • Fichier surveillé : {}", source_path_str);
    println!("   • Intervalle        : {}s", interval_secs);
    println!("   • Appuyez sur Ctrl+C pour arrêter.\n");

    let mut last_mtime = get_mtime(source_path).unwrap_or(SystemTime::UNIX_EPOCH);

    // Première exécution immédiate
    run_pipeline_once(config_path);

    loop {
        std::thread::sleep(Duration::from_secs(interval_secs));

        match get_mtime(source_path) {
            Ok(current_mtime) => {
                if current_mtime != last_mtime {
                    println!(
                        "\n🔄 [WATCH] Changement détecté dans \"{}\" — relancement…",
                        source_path_str
                    );
                    run_pipeline_once(config_path);
                    last_mtime = current_mtime;
                } else {
                    println!("⏳ [WATCH] Pas de changement.");
                }
            }
            Err(e) => eprintln!("⚠️  [WATCH] Impossible de lire le fichier source : {:#}", e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_get_mtime_existing_file() {
        let f = NamedTempFile::new().unwrap();
        assert!(get_mtime(f.path()).is_ok());
    }

    #[test]
    fn test_get_mtime_missing_file() {
        let result = get_mtime(Path::new("/tmp/__datapipe_inexistant__.xyz"));
        assert!(result.is_err());
    }

    #[test]
    fn test_mtime_readable_after_write() {
        let mut f = NamedTempFile::new().unwrap();
        let before = get_mtime(f.path()).unwrap();
        std::thread::sleep(Duration::from_millis(50));
        f.write_all(b"update").unwrap();
        f.flush().unwrap();
        let after = get_mtime(f.path()).unwrap();
        assert!(after >= before);
    }
}