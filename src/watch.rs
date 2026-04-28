#![allow(dead_code)]

use std::path::Path;
use std::fs;
use std::time::SystemTime;
use std::thread;
use std::time::Duration;
use anyhow::Result;

pub fn watch_mode(config_path: &Path, interval_secs: u64) -> Result<()> {
    println!("👀 Mode watch activé - Surveillance toutes les {}s", interval_secs);
    
    let mut last_modified = get_file_mtime(config_path)?;
    
    loop {
        thread::sleep(Duration::from_secs(interval_secs));
        
        match get_file_mtime(config_path) {
            Ok(current) => {
                if current != last_modified {
                    println!("[WATCH] Changement détecté, relancement du pipeline...");
                    // Ici on appellerait run_pipeline() mais c'est optionnel pour le MVP
                    last_modified = current;
                }
            }
            Err(_) => {
                eprintln!("[WATCH] Erreur: impossible de vérifier le fichier config");
            }
        }
    }
}

fn get_file_mtime(path: &Path) -> Result<SystemTime> {
    let metadata = fs::metadata(path)?;
    Ok(metadata.modified()?)
}