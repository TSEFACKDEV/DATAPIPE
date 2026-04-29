// =============================================================================
// src/stats.rs
// Auteur  : DONFACK (#08)
// Rôle    : Collecte et affiche les statistiques complètes d'exécution
//           du pipeline ETL DataPipe.
//
// Ce module est le "tableau de bord" du pipeline. Il mesure et rapporte :
//   - Les compteurs de records (lus, transformés, filtrés, écrits)
//   - Le temps d'exécution et le débit
//   - Les taux de filtrage et d'erreur
//   - L'horodatage de début et de fin
//   - Les infos sur la source et la destination
//   - Le nombre de transformations appliquées
//   - Les records perdus (anomalies)
//   - Les statistiques par colonne (nulls, min, max, moyenne)
// =============================================================================

use std::collections::HashMap;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

// --- Statistiques par colonne ------------------------------------------------
//
// Pour chaque colonne numérique du pipeline, on collecte :
//   - count    : nombre total de valeurs non-nulles rencontrées
//   - null_count : nombre de valeurs nulles (champs vides)
//   - min      : valeur minimale observée
//   - max      : valeur maximale observée
//   - sum      : somme de toutes les valeurs (pour calculer la moyenne)
//
// Ces stats permettent de détecter des anomalies :
//   ex: une colonne "age" avec un min négatif ou un max irréaliste.
//
#[derive(Debug, Default, Clone)]
pub struct ColumnStats {
    /// Nombre de valeurs non-nulles dans cette colonne
    pub count: u64,

    /// Nombre de valeurs nulles/vides dans cette colonne
    pub null_count: u64,

    /// Valeur numérique minimale observée (None si aucune valeur numérique)
    pub min: Option<f64>,

    /// Valeur numérique maximale observée (None si aucune valeur numérique)
    pub max: Option<f64>,

    /// Somme de toutes les valeurs numériques (pour calculer la moyenne)
    pub sum: f64,
}

impl ColumnStats {
    /// Met à jour les stats d'une colonne avec une nouvelle valeur numérique.
    /// Appelé pour chaque valeur f64 rencontrée dans la colonne.
    #[allow(dead_code)]
    pub fn update_numeric(&mut self, val: f64) {
        self.count += 1;
        self.sum += val;

        // Met à jour le minimum : on garde le plus petit
        self.min = Some(match self.min {
            Some(current_min) => current_min.min(val),
            None => val,
        });

        // Met à jour le maximum : on garde le plus grand
        self.max = Some(match self.max {
            Some(current_max) => current_max.max(val),
            None => val,
        });
    }

    /// Enregistre une valeur nulle dans cette colonne.
    #[allow(dead_code)]
    pub fn record_null(&mut self) {
        self.null_count += 1;
    }

    /// Calcule la moyenne des valeurs numériques de cette colonne.
    /// Retourne None si aucune valeur numérique n'a été rencontrée.
    pub fn average(&self) -> Option<f64> {
        if self.count > 0 {
            Some(self.sum / self.count as f64)
        } else {
            None
        }
    }
}

// --- Structure principale des statistiques -----------------------------------
//
// ExecutionStats est le point central de collecte de toutes les métriques.
// Elle est créée au démarrage du pipeline (new()) et arrêtée à la fin (stop()).
// La méthode print_report() affiche ensuite un rapport complet et lisible.
//
#[derive(Debug)]
pub struct ExecutionStats {
    // ── Compteurs de records ─────────────────────────────────────────────────

    /// Nombre total de records lus depuis la source
    pub records_read: u64,

    /// Nombre total de records ayant subi au moins une transformation
    pub records_transformed: u64,

    /// Nombre de records éliminés par une transformation de type "filter"
    pub records_filtered: u64,

    /// Nombre de records écrits avec succès dans la destination
    pub records_written: u64,

    /// Nombre total d'erreurs rencontrées (lecture + transformation + écriture)
    pub errors_encountered: u64,

    // ── Mesure du temps ──────────────────────────────────────────────────────

    /// Instant de démarrage du pipeline (utilisé pour calculer la durée)
    pub start_time: Option<Instant>,

    /// Durée totale d'exécution en millisecondes (calculée par stop())
    pub duration_ms: u64,

    /// Horodatage Unix de démarrage (secondes depuis 1970-01-01)
    /// Utilisé pour afficher l'heure de début en format lisible
    pub start_timestamp: u64,

    /// Horodatage Unix de fin (secondes depuis 1970-01-01)
    /// Calculé lors de l'appel à stop()
    pub end_timestamp: u64,

    // ── Informations sur le pipeline ─────────────────────────────────────────

    /// Chemin du fichier source (ex: "data/etudiants.csv")
    /// Renseigné par pipeline.rs après chargement de la config
    pub source_path: String,

    /// Format du fichier source (ex: "csv", "json", "delimited")
    pub source_format: String,

    /// Chemin du fichier de destination (ex: "output/result.json")
    pub destination_path: String,

    /// Format du fichier de destination (ex: "csv", "json", "jsonl")
    pub destination_format: String,

    /// Nombre de transformations configurées dans le pipeline TOML
    /// (rename, filter, cast, compute, drop, etc.)
    pub transforms_count: usize,

    // ── Statistiques par colonne ─────────────────────────────────────────────

    /// Map nom_colonne → statistiques de cette colonne
    /// Permet de détecter des anomalies colonne par colonne
    pub column_stats: HashMap<String, ColumnStats>,
}

impl Default for ExecutionStats {
    fn default() -> Self {
        ExecutionStats {
            records_read: 0,
            records_transformed: 0,
            records_filtered: 0,
            records_written: 0,
            errors_encountered: 0,
            start_time: None,
            duration_ms: 0,
            start_timestamp: 0,
            end_timestamp: 0,
            source_path: String::new(),
            source_format: String::new(),
            destination_path: String::new(),
            destination_format: String::new(),
            transforms_count: 0,
            column_stats: HashMap::new(),
        }
    }
}

impl ExecutionStats {
    /// Crée un nouveau compteur de stats et démarre le chronomètre.
    /// À appeler au tout début du pipeline, avant toute opération.
    pub fn new() -> Self {
        // On capture l'horodatage Unix pour l'heure de début
        let start_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        ExecutionStats {
            start_time: Some(Instant::now()),
            start_timestamp,
            ..Default::default()
        }
    }

    /// Arrête le chronomètre et enregistre l'heure de fin.
    /// À appeler juste avant d'afficher le rapport.
    pub fn stop(&mut self) {
        // Calcule la durée en millisecondes depuis le démarrage
        if let Some(start) = self.start_time {
            self.duration_ms = start.elapsed().as_millis() as u64;
        }

        // Capture l'horodatage Unix de fin
        self.end_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
    }

    /// Met à jour les statistiques d'une colonne avec une valeur numérique.
    ///
    /// # Arguments
    /// * `col` - Nom de la colonne (ex: "age", "prix")
    /// * `val` - Valeur numérique à enregistrer
    ///
    /// À appeler dans pipeline.rs pour chaque champ numérique d'un record.
    #[allow(dead_code)]
    pub fn update_column_numeric(&mut self, col: &str, val: f64) {
        self.column_stats
            .entry(col.to_string())
            .or_default()
            .update_numeric(val);
    }

    /// Enregistre une valeur nulle dans une colonne.
    ///
    /// # Arguments
    /// * `col` - Nom de la colonne qui contient une valeur nulle
    ///
    /// Utile pour détecter les colonnes avec beaucoup de données manquantes.
    #[allow(dead_code)]
    pub fn record_column_null(&mut self, col: &str) {
        self.column_stats
            .entry(col.to_string())
            .or_default()
            .record_null();
    }

    /// Convertit un horodatage Unix en chaîne lisible (format local simplifié).
    ///
    /// # Arguments
    /// * `timestamp` - Secondes depuis UNIX_EPOCH
    ///
    /// Retourne une chaîne du type "2025-04-27 14:32:05 UTC"
    fn format_timestamp(timestamp: u64) -> String {
        if timestamp == 0 {
            return "N/A".to_string();
        }

        // Calcul manuel de la date à partir du timestamp Unix
        // On évite une dépendance externe (chrono) pour rester léger
        let secs = timestamp;
        let minutes = secs / 60;
        let hours = minutes / 60;
        let days_total = hours / 24;

        let s = secs % 60;
        let m = minutes % 60;
        let h = hours % 24;

        // Calcul de l'année et du jour dans l'année
        let mut year = 1970u64;
        let mut days_remaining = days_total;
        loop {
            let days_in_year = if Self::is_leap_year(year) { 366 } else { 365 };
            if days_remaining < days_in_year {
                break;
            }
            days_remaining -= days_in_year;
            year += 1;
        }

        // Calcul du mois et du jour
        let months_days: [u64; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
        let mut month = 1u64;
        for (i, &days_in_month) in months_days.iter().enumerate() {
            let d = if i == 1 && Self::is_leap_year(year) {
                29
            } else {
                days_in_month
            };
            if days_remaining < d {
                break;
            }
            days_remaining -= d;
            month += 1;
        }
        let day = days_remaining + 1;

        format!(
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02} UTC",
            year, month, day, h, m, s
        )
    }

    /// Vérifie si une année est bissextile.
    fn is_leap_year(year: u64) -> bool {
        (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
    }

    /// Calcule le nombre de records "perdus" :
    /// records lus - records écrits - records filtrés - erreurs.
    ///
    /// Un record perdu est un record qui n'a pas été filtré intentionnellement
    /// et qui n'a pas non plus été écrit : c'est une anomalie à investiguer.
    fn records_perdus(&self) -> i64 {
        self.records_read as i64
            - self.records_written as i64
            - self.records_filtered as i64
            - self.errors_encountered as i64
    }

    /// Affiche le rapport d'exécution complet dans la console.
    ///
    /// Le rapport est structuré en sections :
    ///   1. Informations sur le pipeline (source, destination, transformations)
    ///   2. Horodatage (début, fin, durée)
    ///   3. Compteurs de records
    ///   4. Taux calculés (filtrage, écriture, erreur)
    ///   5. Débit et performance
    ///   6. Anomalies détectées (records perdus)
    ///   7. Statistiques par colonne (si disponibles)
    ///   8. Statut final
    pub fn print_report(&self) {
        let sep_double = "═".repeat(52);
        let sep_simple = "─".repeat(52);

        // ── En-tête ──────────────────────────────────────────────────────────
        println!("\n{}", sep_double);
        println!("         RAPPORT D'EXÉCUTION — DATAPIPE");
        println!("{}", sep_double);

        // ── Section 1 : Informations sur le pipeline ─────────────────────────
        //
        // Affiche d'où viennent les données, où elles vont,
        // et combien de transformations ont été appliquées.
        //
        println!("     PIPELINE");
        println!("{}", sep_simple);

        if !self.source_path.is_empty() {
            println!(
                "    Source        : {} ({})",
                self.source_path, self.source_format
            );
        }

        if !self.destination_path.is_empty() {
            println!(
                "    Destination   : {} ({})",
                self.destination_path, self.destination_format
            );
        }

        // Nombre de transformations configurées dans le fichier TOML
        println!(
            "    Transformations configurées : {}",
            self.transforms_count
        );

        println!("{}", sep_simple);

        // ── Section 2 : Horodatage ────────────────────────────────────────────
        //
        // Affiche l'heure de début, l'heure de fin et la durée totale.
        // Utile pour retrouver dans les logs quand un pipeline s'est exécuté.
        //
        println!("     HORODATAGE");
        println!("{}", sep_simple);

        println!(
            "    Début         : {}",
            Self::format_timestamp(self.start_timestamp)
        );
        println!(
            "    Fin           : {}",
            Self::format_timestamp(self.end_timestamp)
        );

        // Affichage intelligent de la durée : ms ou secondes selon la valeur
        if self.duration_ms < 1000 {
            println!("    Durée         : {}ms", self.duration_ms);
        } else {
            println!(
                "    Durée         : {:.2}s",
                self.duration_ms as f64 / 1000.0
            );
        }

        println!("{}", sep_simple);

        // ── Section 3 : Compteurs de records ─────────────────────────────────
        //
        // Vue d'ensemble du flux de données à travers le pipeline.
        // Permet de vérifier que tous les records ont bien été traités.
        //
        println!("     RECORDS");
        println!("{}", sep_simple);

        println!("    Lus              : {}", self.records_read);
        println!("    Transformés      : {}", self.records_transformed);
        println!("    Filtrés           : {}", self.records_filtered);
        println!("    Écrits            : {}", self.records_written);

        // Erreurs : on distingue "aucune erreur" d'un nombre d'erreurs
        println!("    Erreurs           : {}", self.errors_encountered);

        println!("{}", sep_simple);

        // ── Section 4 : Taux calculés ─────────────────────────────────────────
        //
        // Ces pourcentages donnent une vision rapide de la qualité du pipeline :
        //   - Taux de filtrage élevé → beaucoup de données écartées (normal ou suspect ?)
        //   - Taux d'erreur élevé   → problème de qualité des données source
        //   - Taux d'écriture faible → peu de données arrivent à destination
        //
        if self.records_read > 0 {
            println!("     TAUX");
            println!("{}", sep_simple);

            let taux_filtrage =
                (self.records_filtered as f64 / self.records_read as f64) * 100.0;
            let taux_ecriture =
                (self.records_written as f64 / self.records_read as f64) * 100.0;
            let taux_erreur =
                (self.errors_encountered as f64 / self.records_read as f64) * 100.0;

            // Pourcentage de records éliminés par les filtres
            println!("    Taux de filtrage  : {:.1}%", taux_filtrage);

            // Pourcentage de records ayant atteint la destination
            println!("    Taux d'écriture   : {:.1}%", taux_ecriture);

            // Pourcentage d'erreurs — idéalement doit être 0%
            println!("    Taux d'erreur     : {:.2}%", taux_erreur);

            println!("{}", sep_simple);
        }

        // ── Section 5 : Performance et débit ─────────────────────────────────
        //
        // Le débit mesure combien de records sont traités par seconde.
        // Un débit très faible peut indiquer un goulot d'étranglement
        // (fichier trop grand, transformation lente, disque lent, etc.)
        //
        println!("     PERFORMANCE");
        println!("{}", sep_simple);

        let debit = if self.duration_ms > 0 {
            (self.records_written as f64 / self.duration_ms as f64) * 1000.0
        } else {
            self.records_written as f64
        };

        println!("    Débit             : {:.0} records/s", debit);

        // Estimation du volume : chaque record ≈ 256 octets en mémoire (estimation)
        let volume_kb = (self.records_read as f64 * 256.0) / 1024.0;
        if volume_kb < 1024.0 {
            println!("    Volume estimé     : {:.1} Ko", volume_kb);
        } else {
            println!("    Volume estimé     : {:.2} Mo", volume_kb / 1024.0);
        }

        println!("{}", sep_simple);

        // ── Section 6 : Détection d'anomalies ────────────────────────────────
        //
        // Les "records perdus" sont des records qui ne correspondent à aucune
        // des catégories connues : ni écrits, ni filtrés, ni en erreur.
        // Leur présence indique un bug ou un cas non géré dans le pipeline.
        //
        let perdus = self.records_perdus();
        if perdus != 0 {
            println!("     ANOMALIES DÉTECTÉES");
            println!("{}", sep_simple);
            println!(
                "    Records perdus    : {} (ni écrits, ni filtrés, ni en erreur)",
                perdus.abs()
            );
            println!("      → Vérifier la logique du pipeline !");
            println!("{}", sep_simple);
        }

        // ── Section 7 : Statistiques par colonne ─────────────────────────────
        //
        // Pour chaque colonne ayant des données numériques ou des nulls,
        // on affiche un résumé statistique :
        //   - count      : combien de valeurs non-nulles
        //   - null_count : combien de valeurs manquantes
        //   - min / max  : étendue des valeurs
        //   - moyenne    : valeur centrale
        //
        // Ces infos permettent de détecter rapidement :
        //   - Des colonnes majoritairement vides (données manquantes)
        //   - Des valeurs aberrantes (min ou max irréalistes)
        //   - Des colonnes constantes (min == max)
        //
        if !self.column_stats.is_empty() {
            println!("     STATISTIQUES PAR COLONNE");
            println!("{}", sep_simple);

            // On trie les colonnes par nom pour un affichage cohérent
            let mut colonnes: Vec<(&String, &ColumnStats)> =
                self.column_stats.iter().collect();
            colonnes.sort_by_key(|(name, _)| name.as_str());

            for (nom, stats) in colonnes {
                println!("    Colonne : \"{}\"", nom);

                // Nombre de valeurs non-nulles rencontrées
                println!("      • Valeurs non-nulles  : {}", stats.count);

                // Nombre de valeurs nulles/manquantes
                if stats.null_count > 0 {
                    println!("      • Valeurs nulles      : {} [WARN]", stats.null_count);
                } else {
                    println!("      • Valeurs nulles      : 0 [OK]");
                }

                // Statistiques numériques (min, max, moyenne) si disponibles
                if let (Some(min), Some(max)) = (stats.min, stats.max) {
                    println!("      • Min                 : {:.4}", min);
                    println!("      • Max                 : {:.4}", max);

                    // Alerte si min == max : colonne peut-être constante
                    if (max - min).abs() < f64::EPSILON {
                        println!("      • [WARN]  Min == Max : colonne potentiellement constante");
                    }
                }

                if let Some(avg) = stats.average() {
                    println!("      • Moyenne             : {:.4}", avg);
                }

                println!();
            }
        }

        // ── Section 8 : Statut final ──────────────────────────────────────────
        //
        // Résumé en une ligne de l'issue du pipeline.
        // Permet de savoir d'un coup d'œil si tout s'est bien passé.
        //
        println!("{}", sep_double);

        if self.errors_encountered == 0 && perdus == 0 {
            println!("    STATUT : SUCCÈS — pipeline terminé sans erreur ni anomalie");
        } else if self.errors_encountered > 0 && perdus == 0 {
            println!(
                "  [ERREUR]  STATUT : TERMINÉ AVEC {} ERREUR(S)",
                self.errors_encountered
            );
        } else if self.errors_encountered == 0 && perdus != 0 {
            println!(
                "  [WARN]   STATUT : TERMINÉ — {} record(s) perdu(s) détecté(s)",
                perdus.abs()
            );
        } else {
            println!(
                "  [ERREUR]  STATUT : TERMINÉ AVEC {} ERREUR(S) ET {} ANOMALIE(S)",
                self.errors_encountered,
                perdus.abs()
            );
        }

        println!("{}\n", sep_double);
    }
}

// =============================================================================
// TESTS UNITAIRES
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_demarre_chronometre() {
        let stats = ExecutionStats::new();
        assert!(stats.start_time.is_some());
        assert!(stats.start_timestamp > 0);
        assert_eq!(stats.records_read, 0);
        assert_eq!(stats.records_written, 0);
    }

    #[test]
    fn test_stop_calcule_duree_et_fin() {
        let mut stats = ExecutionStats::new();
        std::thread::sleep(std::time::Duration::from_millis(5));
        stats.stop();
        assert!(stats.duration_ms > 0);
        assert!(stats.end_timestamp >= stats.start_timestamp);
    }

    #[test]
    fn test_records_perdus_zero() {
        let mut stats = ExecutionStats::new();
        stats.records_read = 100;
        stats.records_written = 80;
        stats.records_filtered = 15;
        stats.errors_encountered = 5;
        // 100 - 80 - 15 - 5 = 0 → aucun record perdu
        assert_eq!(stats.records_perdus(), 0);
    }

    #[test]
    fn test_records_perdus_detectes() {
        let mut stats = ExecutionStats::new();
        stats.records_read = 100;
        stats.records_written = 70;
        stats.records_filtered = 10;
        stats.errors_encountered = 5;
        // 100 - 70 - 10 - 5 = 15 records perdus
        assert_eq!(stats.records_perdus(), 15);
    }

    #[test]
    fn test_column_stats_update_numeric() {
        let mut stats = ExecutionStats::new();
        stats.update_column_numeric("age", 25.0);
        stats.update_column_numeric("age", 30.0);
        stats.update_column_numeric("age", 20.0);

        let col = stats.column_stats.get("age").unwrap();
        assert_eq!(col.count, 3);
        assert_eq!(col.min, Some(20.0));
        assert_eq!(col.max, Some(30.0));
        assert_eq!(col.average(), Some(25.0));
    }

    #[test]
    fn test_column_stats_null() {
        let mut stats = ExecutionStats::new();
        stats.record_column_null("email");
        stats.record_column_null("email");

        let col = stats.column_stats.get("email").unwrap();
        assert_eq!(col.null_count, 2);
        assert_eq!(col.count, 0);
    }

    #[test]
    fn test_print_report_complet() {
        let mut stats = ExecutionStats::new();
        stats.source_path = "data/test.csv".to_string();
        stats.source_format = "csv".to_string();
        stats.destination_path = "output/result.json".to_string();
        stats.destination_format = "json".to_string();
        stats.transforms_count = 3;
        stats.records_read = 100;
        stats.records_transformed = 100;
        stats.records_filtered = 10;
        stats.records_written = 88;
        stats.errors_encountered = 2;
        stats.update_column_numeric("age", 20.0);
        stats.update_column_numeric("age", 45.0);
        stats.record_column_null("email");
        stats.stop();
        // Doit s'exécuter sans panique
        stats.print_report();
    }

    #[test]
    fn test_format_timestamp_non_zero() {
        // Un timestamp valide doit retourner une chaîne non vide et non "N/A"
        let result = ExecutionStats::format_timestamp(1_700_000_000);
        assert_ne!(result, "N/A");
        assert!(result.contains("UTC"));
    }

    #[test]
    fn test_format_timestamp_zero() {
        // Un timestamp à 0 doit retourner "N/A"
        assert_eq!(ExecutionStats::format_timestamp(0), "N/A");
    }
}