use std::path::Path;

pub fn watch_mode(config_path: &Path, interval_secs: u64) -> anyhow::Result<()> {
    // TODO: Implémenter le mode watch (NJOH #10)
    // 1. Vérifier le timestamp du fichier source
    // 2. Boucler avec sleep
    // 3. Relancer le pipeline si changement détecté
    println!("👀 Mode watch activé - Surveillance toutes les {}s", interval_secs);
    todo!("Implémenter watch_mode")
}