// =============================================================================
// src/report.rs
// Auteur  : DONFACK (#08)
// Rôle    : Génère un rapport HTML visuel et professionnel à la fin de
//           chaque exécution du pipeline ETL DataPipe.
//
// Le rapport HTML contient :
//   1. Un en-tête avec le nom du pipeline et l'horodatage
//   2. Des cartes de métriques clés (records lus, écrits, filtrés, erreurs)
//   3. Une section performance (durée, débit, volume)
//   4. Les informations sur la source et la destination
//   5. Les statistiques par colonne (min, max, moyenne, nulls)
//   6. Un aperçu des données (preview des premiers records)
//   7. Un statut final coloré (succès ou erreur)
//
// Le fichier HTML généré est autonome (CSS intégré, pas de dépendance externe)
// et peut être ouvert directement dans n'importe quel navigateur.
// =============================================================================

use crate::reader::Record;
use crate::stats::ExecutionStats;
use anyhow::Result;
use std::fs;
use std::path::Path;

// --- Fonction principale de génération du rapport ----------------------------
//
// Génère un fichier HTML complet à partir des statistiques d'exécution
// et d'un aperçu des données traitées.
//
// # Arguments
// * `stats`       - Statistiques complètes collectées pendant l'exécution
// * `preview`     - Slice des premiers records traités (aperçu des données)
// * `output_path` - Chemin du fichier HTML à créer (ex: "reports/rapport.html")
//
// # Exemple d'appel dans pipeline.rs :
//   generate_html_report(&stats, &preview_records, "reports/rapport.html")?;
//
pub fn generate_html_report(
    stats: &ExecutionStats,
    preview: &[Record],
    output_path: &str,
) -> Result<()> {
    // Créer le dossier parent si nécessaire
    // Ex: si output_path = "reports/rapport.html", on crée "reports/"
    if let Some(parent) = Path::new(output_path).parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }

    // Construire le contenu HTML complet
    let html = build_html(stats, preview);

    // Écrire le fichier HTML sur le disque
    fs::write(output_path, &html)?;

    println!("📄 Rapport HTML généré : {}", output_path);
    Ok(())
}

// --- Construction du HTML complet --------------------------------------------
//
// Assemble toutes les sections HTML en un document complet et autonome.
// Le CSS est intégré directement dans le <style> pour éviter toute
// dépendance externe — le fichier fonctionne hors connexion.
//
fn build_html(stats: &ExecutionStats, preview: &[Record]) -> String {
    let css = build_css();
    let header = build_header(stats);
    let cards = build_metric_cards(stats);
    let performance = build_performance_section(stats);
    let pipeline_info = build_pipeline_info(stats);
    let column_stats = build_column_stats(stats);
    let preview_table = build_preview_table(preview);
    let status_banner = build_status_banner(stats);
    let footer = build_footer();

    format!(
        r#"<!DOCTYPE html>
<html lang="fr">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>DataPipe — Rapport d'exécution</title>
    <style>{css}</style>
</head>
<body>
    <div class="container">
        {header}
        {status_banner}
        {cards}
        {performance}
        {pipeline_info}
        {column_stats}
        {preview_table}
        {footer}
    </div>
</body>
</html>"#
    )
}

// --- CSS intégré -------------------------------------------------------------
//
// Design moderne avec :
//   - Palette de couleurs sombre et professionnelle
//   - Cartes avec ombres et coins arrondis
//   - Animations subtiles au survol
//   - Typographie claire et lisible
//   - Responsive (s'adapte aux petits écrans)
//
fn build_css() -> String {
    r#"
        /* ── Reset et base ───────────────────────────────────────────── */
        *, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }

        /* ── Variables de couleurs ───────────────────────────────────── */
        :root {
            --bg-primary:    #0f1117;
            --bg-secondary:  #1a1d27;
            --bg-card:       #20243a;
            --bg-card-hover: #252942;
            --accent-blue:   #4f8ef7;
            --accent-green:  #22c55e;
            --accent-orange: #f59e0b;
            --accent-red:    #ef4444;
            --accent-purple: #a855f7;
            --accent-cyan:   #06b6d4;
            --text-primary:  #e2e8f0;
            --text-secondary:#94a3b8;
            --text-muted:    #64748b;
            --border:        #2d3250;
            --border-light:  #363d5e;
            --shadow:        0 4px 24px rgba(0,0,0,0.4);
            --shadow-hover:  0 8px 32px rgba(79,142,247,0.15);
            --radius:        12px;
            --radius-sm:     8px;
        }

        /* ── Corps de la page ────────────────────────────────────────── */
        body {
            font-family: 'Segoe UI', system-ui, -apple-system, sans-serif;
            background: var(--bg-primary);
            color: var(--text-primary);
            line-height: 1.6;
            min-height: 100vh;
            padding: 24px 16px;
        }

        /* ── Conteneur principal ─────────────────────────────────────── */
        .container {
            max-width: 1100px;
            margin: 0 auto;
            display: flex;
            flex-direction: column;
            gap: 28px;
        }

        /* ── En-tête ─────────────────────────────────────────────────── */
        .header {
            background: linear-gradient(135deg, #1a1d27 0%, #20243a 50%, #1e2235 100%);
            border: 1px solid var(--border-light);
            border-radius: var(--radius);
            padding: 36px 40px;
            display: flex;
            align-items: center;
            justify-content: space-between;
            box-shadow: var(--shadow);
            position: relative;
            overflow: hidden;
        }

        /* Ligne décorative en haut de l'en-tête */
        .header::before {
            content: '';
            position: absolute;
            top: 0; left: 0; right: 0;
            height: 3px;
            background: linear-gradient(90deg, var(--accent-blue), var(--accent-purple), var(--accent-cyan));
        }

        .header-left h1 {
            font-size: 2rem;
            font-weight: 700;
            background: linear-gradient(135deg, var(--accent-blue), var(--accent-cyan));
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
            background-clip: text;
            letter-spacing: -0.5px;
        }

        .header-left .subtitle {
            color: var(--text-secondary);
            font-size: 0.9rem;
            margin-top: 4px;
        }

        .header-right {
            text-align: right;
        }

        .header-right .timestamp {
            color: var(--text-secondary);
            font-size: 0.85rem;
            font-family: 'Courier New', monospace;
        }

        .badge {
            display: inline-block;
            padding: 4px 12px;
            border-radius: 20px;
            font-size: 0.78rem;
            font-weight: 600;
            letter-spacing: 0.5px;
            margin-top: 8px;
        }

        .badge-success { background: rgba(34,197,94,0.15); color: var(--accent-green); border: 1px solid rgba(34,197,94,0.3); }
        .badge-error   { background: rgba(239,68,68,0.15);  color: var(--accent-red);   border: 1px solid rgba(239,68,68,0.3); }
        .badge-warning { background: rgba(245,158,11,0.15); color: var(--accent-orange); border: 1px solid rgba(245,158,11,0.3); }

        /* ── Bannière de statut ───────────────────────────────────────── */
        .status-banner {
            border-radius: var(--radius);
            padding: 18px 28px;
            display: flex;
            align-items: center;
            gap: 14px;
            font-size: 1rem;
            font-weight: 500;
            border: 1px solid;
        }

        .status-banner.success {
            background: rgba(34,197,94,0.08);
            border-color: rgba(34,197,94,0.25);
            color: var(--accent-green);
        }

        .status-banner.error {
            background: rgba(239,68,68,0.08);
            border-color: rgba(239,68,68,0.25);
            color: var(--accent-red);
        }

        .status-banner.warning {
            background: rgba(245,158,11,0.08);
            border-color: rgba(245,158,11,0.25);
            color: var(--accent-orange);
        }

        .status-icon { font-size: 1.5rem; }

        /* ── Grille de cartes ────────────────────────────────────────── */
        .cards-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 16px;
        }

        .card {
            background: var(--bg-card);
            border: 1px solid var(--border);
            border-radius: var(--radius);
            padding: 24px 20px;
            display: flex;
            flex-direction: column;
            gap: 10px;
            box-shadow: var(--shadow);
            transition: transform 0.2s, box-shadow 0.2s, border-color 0.2s;
            position: relative;
            overflow: hidden;
        }

        .card:hover {
            transform: translateY(-3px);
            box-shadow: var(--shadow-hover);
            border-color: var(--border-light);
            background: var(--bg-card-hover);
        }

        /* Barre colorée en haut de chaque carte */
        .card::before {
            content: '';
            position: absolute;
            top: 0; left: 0; right: 0;
            height: 2px;
        }

        .card-blue::before   { background: var(--accent-blue); }
        .card-green::before  { background: var(--accent-green); }
        .card-orange::before { background: var(--accent-orange); }
        .card-red::before    { background: var(--accent-red); }
        .card-purple::before { background: var(--accent-purple); }
        .card-cyan::before   { background: var(--accent-cyan); }

        .card-icon { font-size: 1.6rem; }

        .card-label {
            font-size: 0.78rem;
            color: var(--text-muted);
            text-transform: uppercase;
            letter-spacing: 0.8px;
            font-weight: 600;
        }

        .card-value {
            font-size: 2.2rem;
            font-weight: 700;
            color: var(--text-primary);
            line-height: 1;
        }

        .card-blue   .card-value { color: var(--accent-blue); }
        .card-green  .card-value { color: var(--accent-green); }
        .card-orange .card-value { color: var(--accent-orange); }
        .card-red    .card-value { color: var(--accent-red); }
        .card-purple .card-value { color: var(--accent-purple); }
        .card-cyan   .card-value { color: var(--accent-cyan); }

        .card-sub {
            font-size: 0.78rem;
            color: var(--text-muted);
        }

        /* ── Sections génériques ─────────────────────────────────────── */
        .section {
            background: var(--bg-card);
            border: 1px solid var(--border);
            border-radius: var(--radius);
            overflow: hidden;
            box-shadow: var(--shadow);
        }

        .section-header {
            padding: 18px 24px;
            background: rgba(255,255,255,0.02);
            border-bottom: 1px solid var(--border);
            display: flex;
            align-items: center;
            gap: 10px;
        }

        .section-header h2 {
            font-size: 1rem;
            font-weight: 600;
            color: var(--text-primary);
        }

        .section-header .section-icon { font-size: 1.1rem; }

        .section-body {
            padding: 24px;
        }

        /* ── Grille d'informations ───────────────────────────────────── */
        .info-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
            gap: 16px;
        }

        .info-item {
            display: flex;
            flex-direction: column;
            gap: 4px;
        }

        .info-label {
            font-size: 0.75rem;
            color: var(--text-muted);
            text-transform: uppercase;
            letter-spacing: 0.6px;
            font-weight: 600;
        }

        .info-value {
            font-size: 0.92rem;
            color: var(--text-primary);
            font-family: 'Courier New', monospace;
            background: rgba(255,255,255,0.04);
            padding: 6px 10px;
            border-radius: var(--radius-sm);
            border: 1px solid var(--border);
            word-break: break-all;
        }

        /* ── Barres de progression ───────────────────────────────────── */
        .progress-row {
            display: flex;
            flex-direction: column;
            gap: 6px;
            margin-bottom: 14px;
        }

        .progress-label {
            display: flex;
            justify-content: space-between;
            font-size: 0.83rem;
            color: var(--text-secondary);
        }

        .progress-bar-bg {
            height: 8px;
            background: rgba(255,255,255,0.06);
            border-radius: 4px;
            overflow: hidden;
        }

        .progress-bar-fill {
            height: 100%;
            border-radius: 4px;
            transition: width 0.6s ease;
        }

        .fill-blue   { background: linear-gradient(90deg, var(--accent-blue), var(--accent-cyan)); }
        .fill-green  { background: linear-gradient(90deg, var(--accent-green), #16a34a); }
        .fill-orange { background: linear-gradient(90deg, var(--accent-orange), #d97706); }
        .fill-red    { background: linear-gradient(90deg, var(--accent-red), #dc2626); }

        /* ── Statistiques par colonne ────────────────────────────────── */
        .col-stats-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
            gap: 14px;
        }

        .col-stat-card {
            background: rgba(255,255,255,0.02);
            border: 1px solid var(--border);
            border-radius: var(--radius-sm);
            padding: 16px;
        }

        .col-stat-name {
            font-size: 0.85rem;
            font-weight: 600;
            color: var(--accent-cyan);
            font-family: 'Courier New', monospace;
            margin-bottom: 10px;
            padding-bottom: 8px;
            border-bottom: 1px solid var(--border);
        }

        .col-stat-row {
            display: flex;
            justify-content: space-between;
            font-size: 0.82rem;
            padding: 3px 0;
        }

        .col-stat-key  { color: var(--text-muted); }
        .col-stat-val  { color: var(--text-primary); font-weight: 500; }
        .col-stat-warn { color: var(--accent-orange); font-weight: 600; }

        /* ── Tableau de prévisualisation ─────────────────────────────── */
        .table-wrapper {
            overflow-x: auto;
            border-radius: var(--radius-sm);
        }

        table {
            width: 100%;
            border-collapse: collapse;
            font-size: 0.83rem;
        }

        thead tr {
            background: rgba(79,142,247,0.08);
        }

        thead th {
            padding: 12px 14px;
            text-align: left;
            font-size: 0.75rem;
            font-weight: 600;
            text-transform: uppercase;
            letter-spacing: 0.6px;
            color: var(--accent-blue);
            border-bottom: 2px solid var(--border-light);
            white-space: nowrap;
        }

        tbody tr {
            border-bottom: 1px solid var(--border);
            transition: background 0.15s;
        }

        tbody tr:hover { background: rgba(255,255,255,0.03); }

        tbody tr:last-child { border-bottom: none; }

        tbody td {
            padding: 10px 14px;
            color: var(--text-secondary);
            max-width: 200px;
            overflow: hidden;
            text-overflow: ellipsis;
            white-space: nowrap;
        }

        .null-value {
            color: var(--text-muted);
            font-style: italic;
            font-size: 0.78rem;
        }

        /* ── Pied de page ────────────────────────────────────────────── */
        .footer {
            text-align: center;
            padding: 20px;
            color: var(--text-muted);
            font-size: 0.78rem;
            border-top: 1px solid var(--border);
        }

        .footer span {
            color: var(--accent-blue);
            font-weight: 600;
        }

        /* ── Responsive ──────────────────────────────────────────────── */
        @media (max-width: 640px) {
            .header { flex-direction: column; gap: 16px; text-align: center; }
            .header-right { text-align: center; }
            .header-left h1 { font-size: 1.5rem; }
            .card-value { font-size: 1.8rem; }
            .section-body { padding: 16px; }
        }
    "#.to_string()
}

// --- En-tête du rapport ------------------------------------------------------
//
// Affiche le titre "DataPipe", le nom du pipeline (source → destination)
// et l'horodatage de génération du rapport.
//
fn build_header(stats: &ExecutionStats) -> String {
    let source = if stats.source_path.is_empty() {
        "Source inconnue".to_string()
    } else {
        format!("{} ({})", stats.source_path, stats.source_format)
    };

    let destination = if stats.destination_path.is_empty() {
        "Destination inconnue".to_string()
    } else {
        format!("{} ({})", stats.destination_path, stats.destination_format)
    };

    let timestamp = format_timestamp_html(stats.start_timestamp);

    format!(
        r#"<div class="header">
            <div class="header-left">
                <h1>⚡ DataPipe</h1>
                <div class="subtitle">Rapport d'exécution du pipeline ETL</div>
                <div class="subtitle" style="margin-top:6px; color:#64748b; font-size:0.8rem;">
                    {} → {}
                </div>
            </div>
            <div class="header-right">
                <div class="timestamp">Généré le {}</div>
                <div class="timestamp" style="margin-top:4px;">
                    {} transformation(s) configurée(s)
                </div>
            </div>
        </div>"#,
        escape_html(&source),
        escape_html(&destination),
        timestamp,
        stats.transforms_count,
    )
}

// --- Bannière de statut final ------------------------------------------------
//
// Affiche une bannière verte (succès), rouge (erreurs) ou orange (anomalies)
// selon l'issue de l'exécution du pipeline.
//
fn build_status_banner(stats: &ExecutionStats) -> String {
    let records_perdus = stats.records_read as i64
        - stats.records_written as i64
        - stats.records_filtered as i64
        - stats.errors_encountered as i64;

    let (class, icon, message) = if stats.errors_encountered == 0 && records_perdus == 0 {
        (
            "success",
            "🎉",
            format!(
                "Pipeline terminé avec succès — {} record(s) traité(s) sans erreur ni anomalie",
                stats.records_written
            ),
        )
    } else if stats.errors_encountered > 0 {
        (
            "error",
            "❌",
            format!(
                "Pipeline terminé avec {} erreur(s) — vérifier les logs pour plus de détails",
                stats.errors_encountered
            ),
        )
    } else {
        (
            "warning",
            "⚠️",
            format!(
                "Pipeline terminé — {} record(s) perdu(s) détecté(s), investigation recommandée",
                records_perdus.abs()
            ),
        )
    };

    format!(
        r#"<div class="status-banner {class}">
            <span class="status-icon">{icon}</span>
            <span>{message}</span>
        </div>"#
    )
}

// --- Cartes de métriques clés ------------------------------------------------
//
// Affiche 5 cartes colorées avec les chiffres essentiels :
// records lus, écrits, filtrés, erreurs et durée.
// Chaque carte a une couleur distinctive pour identification rapide.
//
fn build_metric_cards(stats: &ExecutionStats) -> String {
    // Calcul du pourcentage d'écriture pour le sous-titre
    let pct_ecrits = if stats.records_read > 0 {
        format!(
            "{:.1}% des records lus",
            (stats.records_written as f64 / stats.records_read as f64) * 100.0
        )
    } else {
        "0% des records lus".to_string()
    };

    let pct_filtres = if stats.records_read > 0 {
        format!(
            "{:.1}% des records lus",
            (stats.records_filtered as f64 / stats.records_read as f64) * 100.0
        )
    } else {
        "0% des records lus".to_string()
    };

    let duree_affichee = if stats.duration_ms < 1000 {
        format!("{}ms", stats.duration_ms)
    } else {
        format!("{:.2}s", stats.duration_ms as f64 / 1000.0)
    };

    format!(
        r#"<div class="cards-grid">
            <div class="card card-blue">
                <span class="card-icon">📥</span>
                <span class="card-label">Records lus</span>
                <span class="card-value">{}</span>
                <span class="card-sub">depuis la source</span>
            </div>
            <div class="card card-green">
                <span class="card-icon">📤</span>
                <span class="card-label">Records écrits</span>
                <span class="card-value">{}</span>
                <span class="card-sub">{}</span>
            </div>
            <div class="card card-orange">
                <span class="card-icon">🚫</span>
                <span class="card-label">Records filtrés</span>
                <span class="card-value">{}</span>
                <span class="card-sub">{}</span>
            </div>
            <div class="card card-red">
                <span class="card-icon">⚠️</span>
                <span class="card-label">Erreurs</span>
                <span class="card-value">{}</span>
                <span class="card-sub">pendant l'exécution</span>
            </div>
            <div class="card card-purple">
                <span class="card-icon">⏱️</span>
                <span class="card-label">Durée</span>
                <span class="card-value">{}</span>
                <span class="card-sub">temps total</span>
            </div>
        </div>"#,
        stats.records_read,
        stats.records_written,
        pct_ecrits,
        stats.records_filtered,
        pct_filtres,
        stats.errors_encountered,
        duree_affichee,
    )
}

// --- Section performance -----------------------------------------------------
//
// Affiche le débit, le volume estimé et des barres de progression
// pour visualiser la répartition des records (écrits vs filtrés vs erreurs).
//
fn build_performance_section(stats: &ExecutionStats) -> String {
    let debit = if stats.duration_ms > 0 {
        format!(
            "{:.0} records/s",
            (stats.records_written as f64 / stats.duration_ms as f64) * 1000.0
        )
    } else {
        format!("{} records/s", stats.records_written)
    };

    let volume_kb = (stats.records_read as f64 * 256.0) / 1024.0;
    let volume_str = if volume_kb < 1024.0 {
        format!("{:.1} Ko", volume_kb)
    } else {
        format!("{:.2} Mo", volume_kb / 1024.0)
    };

    // Calcul des pourcentages pour les barres de progression
    let (pct_ecrits, pct_filtres, pct_erreurs) = if stats.records_read > 0 {
        (
            ((stats.records_written as f64 / stats.records_read as f64) * 100.0) as u64,
            ((stats.records_filtered as f64 / stats.records_read as f64) * 100.0) as u64,
            ((stats.errors_encountered as f64 / stats.records_read as f64) * 100.0) as u64,
        )
    } else {
        (0, 0, 0)
    };

    format!(
        r#"<div class="section">
            <div class="section-header">
                <span class="section-icon">⚡</span>
                <h2>Performance</h2>
            </div>
            <div class="section-body">
                <div class="info-grid" style="margin-bottom:24px;">
                    <div class="info-item">
                        <span class="info-label">Débit</span>
                        <span class="info-value">{debit}</span>
                    </div>
                    <div class="info-item">
                        <span class="info-label">Volume estimé</span>
                        <span class="info-value">{volume_str}</span>
                    </div>
                    <div class="info-item">
                        <span class="info-label">Début</span>
                        <span class="info-value">{debut}</span>
                    </div>
                    <div class="info-item">
                        <span class="info-label">Fin</span>
                        <span class="info-value">{fin}</span>
                    </div>
                </div>

                <div class="progress-row">
                    <div class="progress-label">
                        <span>📤 Records écrits</span>
                        <span>{pct_ecrits}%</span>
                    </div>
                    <div class="progress-bar-bg">
                        <div class="progress-bar-fill fill-green" style="width:{pct_ecrits}%"></div>
                    </div>
                </div>
                <div class="progress-row">
                    <div class="progress-label">
                        <span>🚫 Records filtrés</span>
                        <span>{pct_filtres}%</span>
                    </div>
                    <div class="progress-bar-bg">
                        <div class="progress-bar-fill fill-orange" style="width:{pct_filtres}%"></div>
                    </div>
                </div>
                <div class="progress-row">
                    <div class="progress-label">
                        <span>⚠️ Taux d'erreur</span>
                        <span>{pct_erreurs}%</span>
                    </div>
                    <div class="progress-bar-bg">
                        <div class="progress-bar-fill fill-red" style="width:{pct_erreurs}%"></div>
                    </div>
                </div>
            </div>
        </div>"#,
        debit = debit,
        volume_str = volume_str,
        debut = format_timestamp_html(stats.start_timestamp),
        fin = format_timestamp_html(stats.end_timestamp),
        pct_ecrits = pct_ecrits,
        pct_filtres = pct_filtres,
        pct_erreurs = pct_erreurs,
    )
}

// --- Section informations pipeline -------------------------------------------
//
// Affiche les métadonnées du pipeline : source, destination, format, transforms.
//
fn build_pipeline_info(stats: &ExecutionStats) -> String {
    let source = if stats.source_path.is_empty() {
        "Non spécifiée".to_string()
    } else {
        stats.source_path.clone()
    };

    let dest = if stats.destination_path.is_empty() {
        "Non spécifiée".to_string()
    } else {
        stats.destination_path.clone()
    };

    format!(
        r#"<div class="section">
            <div class="section-header">
                <span class="section-icon">🗂️</span>
                <h2>Informations pipeline</h2>
            </div>
            <div class="section-body">
                <div class="info-grid">
                    <div class="info-item">
                        <span class="info-label">Fichier source</span>
                        <span class="info-value">{}</span>
                    </div>
                    <div class="info-item">
                        <span class="info-label">Format source</span>
                        <span class="info-value">{}</span>
                    </div>
                    <div class="info-item">
                        <span class="info-label">Fichier destination</span>
                        <span class="info-value">{}</span>
                    </div>
                    <div class="info-item">
                        <span class="info-label">Format destination</span>
                        <span class="info-value">{}</span>
                    </div>
                    <div class="info-item">
                        <span class="info-label">Transformations</span>
                        <span class="info-value">{} configurée(s)</span>
                    </div>
                    <div class="info-item">
                        <span class="info-label">Records transformés</span>
                        <span class="info-value">{}</span>
                    </div>
                </div>
            </div>
        </div>"#,
        escape_html(&source),
        escape_html(&stats.source_format),
        escape_html(&dest),
        escape_html(&stats.destination_format),
        stats.transforms_count,
        stats.records_transformed,
    )
}

// --- Section statistiques par colonne ----------------------------------------
//
// Pour chaque colonne ayant des données collectées, affiche une carte avec
// le nombre de valeurs, les nulls, le min, le max et la moyenne.
// Affiche un avertissement si des valeurs nulles sont détectées.
//
fn build_column_stats(stats: &ExecutionStats) -> String {
    if stats.column_stats.is_empty() {
        return String::new();
    }

    // Trier les colonnes par nom pour un affichage cohérent
    let mut colonnes: Vec<(&String, &crate::stats::ColumnStats)> =
        stats.column_stats.iter().collect();
    colonnes.sort_by_key(|(name, _)| name.as_str());

    let mut cards = String::new();

    for (nom, col) in &colonnes {
        // Avertissement si des nulls sont détectés dans cette colonne
        let null_html = if col.null_count > 0 {
            format!(
                r#"<div class="col-stat-row">
                    <span class="col-stat-key">Valeurs nulles</span>
                    <span class="col-stat-warn">⚠️ {}</span>
                </div>"#,
                col.null_count
            )
        } else {
            format!(
                r#"<div class="col-stat-row">
                    <span class="col-stat-key">Valeurs nulles</span>
                    <span class="col-stat-val">0 ✅</span>
                </div>"#
            )
        };

        // Statistiques numériques (min, max, moyenne) si disponibles
        let num_html = if let (Some(min), Some(max)) = (col.min, col.max) {
            let avg = col.average().unwrap_or(0.0);
            format!(
                r#"<div class="col-stat-row">
                    <span class="col-stat-key">Minimum</span>
                    <span class="col-stat-val">{:.4}</span>
                </div>
                <div class="col-stat-row">
                    <span class="col-stat-key">Maximum</span>
                    <span class="col-stat-val">{:.4}</span>
                </div>
                <div class="col-stat-row">
                    <span class="col-stat-key">Moyenne</span>
                    <span class="col-stat-val">{:.4}</span>
                </div>"#,
                min, max, avg
            )
        } else {
            String::new()
        };

        cards.push_str(&format!(
            r#"<div class="col-stat-card">
                <div class="col-stat-name">📌 {}</div>
                <div class="col-stat-row">
                    <span class="col-stat-key">Valeurs non-nulles</span>
                    <span class="col-stat-val">{}</span>
                </div>
                {}
                {}
            </div>"#,
            escape_html(nom),
            col.count,
            null_html,
            num_html,
        ));
    }

    format!(
        r#"<div class="section">
            <div class="section-header">
                <span class="section-icon">🔬</span>
                <h2>Statistiques par colonne</h2>
            </div>
            <div class="section-body">
                <div class="col-stats-grid">
                    {}
                </div>
            </div>
        </div>"#,
        cards
    )
}

// --- Tableau de prévisualisation des données ---------------------------------
//
// Affiche les N premiers records traités sous forme de tableau HTML.
// Permet de vérifier visuellement que les transformations ont bien été
// appliquées et que les données ont la forme attendue.
//
fn build_preview_table(preview: &[Record]) -> String {
    if preview.is_empty() {
        return String::new();
    }

    // Collecter toutes les colonnes (en-têtes du tableau)
    // On prend les colonnes du premier record comme référence
    let headers: Vec<String> = preview[0].keys().cloned().collect();

    // Construire les en-têtes du tableau
    let headers_html: String = headers
        .iter()
        .map(|h| format!("<th>{}</th>", escape_html(h)))
        .collect();

    // Construire les lignes du tableau
    let rows_html: String = preview
        .iter()
        .map(|record| {
            let cells: String = headers
                .iter()
                .map(|col| {
                    let val = match record.get(col) {
                        Some(serde_json::Value::Null) | None => {
                            r#"<span class="null-value">null</span>"#.to_string()
                        }
                        Some(serde_json::Value::String(s)) => escape_html(s),
                        Some(v) => escape_html(&v.to_string()),
                    };
                    format!("<td>{}</td>", val)
                })
                .collect();
            format!("<tr>{}</tr>", cells)
        })
        .collect();

    format!(
        r#"<div class="section">
            <div class="section-header">
                <span class="section-icon">👁️</span>
                <h2>Aperçu des données ({} record(s))</h2>
            </div>
            <div class="section-body" style="padding:0;">
                <div class="table-wrapper">
                    <table>
                        <thead><tr>{}</tr></thead>
                        <tbody>{}</tbody>
                    </table>
                </div>
            </div>
        </div>"#,
        preview.len(),
        headers_html,
        rows_html,
    )
}

// --- Pied de page ------------------------------------------------------------
fn build_footer() -> String {
    r#"<div class="footer">
        Rapport généré par <span>DataPipe</span> — Groupe 6 · ENSP Yaoundé 2024-2025
    </div>"#
        .to_string()
}

// --- Utilitaire : formatage d'un timestamp Unix ------------------------------
//
// Convertit un timestamp Unix (secondes) en chaîne lisible.
// Implémenté sans dépendance externe pour rester léger.
//
fn format_timestamp_html(timestamp: u64) -> String {
    if timestamp == 0 {
        return "N/A".to_string();
    }

    let secs = timestamp;
    let minutes = secs / 60;
    let hours = minutes / 60;
    let days_total = hours / 24;

    let s = secs % 60;
    let m = minutes % 60;
    let h = hours % 24;

    let mut year = 1970u64;
    let mut days_remaining = days_total;
    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if days_remaining < days_in_year {
            break;
        }
        days_remaining -= days_in_year;
        year += 1;
    }

    let months_days: [u64; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut month = 1u64;
    for (i, &days_in_month) in months_days.iter().enumerate() {
        let d = if i == 1 && is_leap_year(year) { 29 } else { days_in_month };
        if days_remaining < d { break; }
        days_remaining -= d;
        month += 1;
    }
    let day = days_remaining + 1;

    format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02} UTC", year, month, day, h, m, s)
}

fn is_leap_year(year: u64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

// --- Utilitaire : échappement HTML -------------------------------------------
//
// Échappe les caractères spéciaux HTML pour éviter les injections
// et les erreurs d'affichage dans le navigateur.
//
fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

// =============================================================================
// TESTS UNITAIRES
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use crate::stats::ExecutionStats;
    use indexmap::IndexMap;
    use serde_json::json;
    use tempfile::tempdir;

    fn make_stats() -> ExecutionStats {
        let mut s = ExecutionStats::new();
        s.records_read = 100;
        s.records_written = 85;
        s.records_filtered = 12;
        s.records_transformed = 100;
        s.errors_encountered = 3;
        s.source_path = "data/test.csv".to_string();
        s.source_format = "csv".to_string();
        s.destination_path = "output/result.json".to_string();
        s.destination_format = "json".to_string();
        s.transforms_count = 3;
        s.update_column_numeric("age", 20.0);
        s.update_column_numeric("age", 45.0);
        s.record_column_null("email");
        s.stop();
        s
    }

    #[test]
    fn test_generate_html_report_cree_fichier() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("rapport.html");
        let stats = make_stats();

        let result = generate_html_report(&stats, &[], path.to_str().unwrap());
        assert!(result.is_ok());
        assert!(path.exists(), "Le fichier HTML doit être créé");
    }

    #[test]
    fn test_generate_html_report_cree_dossier() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("reports").join("rapport.html");
        let stats = make_stats();

        let result = generate_html_report(&stats, &[], path.to_str().unwrap());
        assert!(result.is_ok());
        assert!(path.exists(), "Le dossier parent doit être créé automatiquement");
    }

    #[test]
    fn test_html_contient_sections_cles() {
        let stats = make_stats();
        let html = build_html(&stats, &[]);

        // Vérifier que les sections essentielles sont présentes
        assert!(html.contains("DataPipe"));
        assert!(html.contains("Records lus"));
        assert!(html.contains("Records écrits"));
        assert!(html.contains("Performance"));
        assert!(html.contains("Informations pipeline"));
    }

    #[test]
    fn test_html_contient_donnees_stats() {
        let stats = make_stats();
        let html = build_html(&stats, &[]);

        // Les valeurs des stats doivent apparaître dans le HTML
        assert!(html.contains("100")); // records_read
        assert!(html.contains("data/test.csv"));
        assert!(html.contains("output/result.json"));
    }

    #[test]
    fn test_preview_table_avec_records() {
        let mut record: Record = IndexMap::new();
        record.insert("nom".to_string(), json!("Alice"));
        record.insert("age".to_string(), json!(30));

        let html = build_preview_table(&[record]);
        assert!(html.contains("nom"));
        assert!(html.contains("Alice"));
        assert!(html.contains("30"));
    }

    #[test]
    fn test_escape_html_caracteres_speciaux() {
        assert_eq!(escape_html("<script>"), "&lt;script&gt;");
        assert_eq!(escape_html("a & b"), "a &amp; b");
        assert_eq!(escape_html("\"texte\""), "&quot;texte&quot;");
    }

    #[test]
    fn test_preview_table_vide_retourne_vide() {
        let html = build_preview_table(&[]);
        assert!(html.is_empty());
    }
}