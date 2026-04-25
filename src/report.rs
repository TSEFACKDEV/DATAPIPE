// src/report.rs
use crate::stats::ExecutionStats;
use crate::reader::Record;

pub fn generate_html_report(
    stats: &ExecutionStats,
    preview: &[Record],
    output_path: &str,
) -> anyhow::Result<()> {
    // TODO: Implémenter le rapport HTML (DONFACK #08)
    // 1. Créer le fichier HTML
    // 2. Ajouter les statistiques avec style
    // 3. Ajouter un aperçu des données
    println!("📄 Rapport HTML généré: {}", output_path);
    Ok(())
}