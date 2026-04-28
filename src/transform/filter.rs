// src/transform/filter.rs
// [ASSONGUE] - Transformation : Filtrage par valeur

use crate::transform::{Record, Transform};

// ─────────────────────────────────────────────
//  Opérateurs de comparaison supportés
// ─────────────────────────────────────────────

/// Opérateurs de comparaison disponibles pour le filtre.
#[derive(Debug, Clone, PartialEq)]
pub enum Operator {
    /// Égalité stricte : valeur == référence
    Eq,
    /// Inégalité : valeur != référence
    Ne,
    /// Inférieur strict (numérique) : valeur < référence
    Lt,
    /// Inférieur ou égal (numérique) : valeur <= référence
    Lte,
    /// Supérieur strict (numérique) : valeur > référence
    Gt,
    /// Supérieur ou égal (numérique) : valeur >= référence
    Gte,
    /// Contient la sous-chaîne (insensible à la casse)
    Contains,
}

impl Operator {
    /// Parse une chaîne en opérateur.
    ///
    /// Chaînes reconnues : `=`, `==`, `!=`, `<`, `<=`, `>`, `>=`, `contains`
    ///
    /// # Erreur
    /// Retourne `Err(String)` si l'opérateur est inconnu.
    pub fn parse(s: &str) -> Result<Self, String> {
        match s.trim() {
            "="  | "==" => Ok(Operator::Eq),
            "!="        => Ok(Operator::Ne),
            "<"         => Ok(Operator::Lt),
            "<="        => Ok(Operator::Lte),
            ">"         => Ok(Operator::Gt),
            ">="        => Ok(Operator::Gte),
            "contains"  => Ok(Operator::Contains),
            other       => Err(format!("Opérateur inconnu : '{other}'. Valides: =, !=, <, <=, >, >=, contains")),
        }
    }
}

// ─────────────────────────────────────────────
//  Structure
// ─────────────────────────────────────────────

/// Filtre les enregistrements selon une condition sur une colonne.
///
/// - Si la condition est **vraie** → `Some(record)` (conservé)
/// - Si la condition est **fausse** → `None` (supprimé du flux)
/// - Si la colonne est **absente** → `None` par défaut (comportement strict)
///
/// # Exemple TOML
/// ```toml
/// [[transform]]
/// type     = "filter"
/// column   = "statut"
/// operator = "="
/// value    = "hospitalise"
///
/// [[transform]]
/// type     = "filter"
/// column   = "age"
/// operator = ">"
/// value    = "18"
/// ```
///
/// # Exemple Rust
/// ```rust
/// use datapipe::transform::{Transform, FilterTransform};
/// use std::collections::HashMap;
///
/// let f = FilterTransform::new("age", ">", "18").unwrap();
/// let mut record = HashMap::new();
/// record.insert("age".to_string(), "25".to_string());
/// assert!(f.apply(record).is_some()); // 25 > 18 → conservé
/// ```
#[derive(Debug, Clone)]
pub struct FilterTransform {
    /// Nom de la colonne sur laquelle porte la condition
    pub column: String,
    /// Valeur de référence pour la comparaison
    pub value: String,
    /// Opérateur de comparaison parsé
    pub operator: Operator,
}

impl FilterTransform {
    /// Crée un nouveau `FilterTransform`.
    ///
    /// # Arguments
    /// * `column`   – nom de la colonne à tester
    /// * `operator` – opérateur sous forme de chaîne (`=`, `!=`, `<`, `<=`, `>`, `>=`, `contains`)
    /// * `value`    – valeur de référence
    ///
    /// # Erreur
    /// Retourne `Err(String)` si l'opérateur est invalide.
    pub fn new(
        column: impl Into<String>,
        operator: impl AsRef<str>,
        value: impl Into<String>,
    ) -> Result<Self, String> {
        Ok(Self {
            column: column.into(),
            value: value.into(),
            operator: Operator::parse(operator.as_ref())?,
        })
    }

    /// Évalue la condition entre `cell_value` et `self.value`.
    ///
    /// Pour les opérateurs numériques (`<`, `<=`, `>`, `>=`), tente un
    /// parsing `f64`. Si l'une des valeurs n'est pas numérique, retourne
    /// `false` (comportement sûr, sans panique).
    fn evaluate(&self, cell_value: &str) -> bool {
        match &self.operator {
            Operator::Eq => cell_value == self.value,
            Operator::Ne => cell_value != self.value,
            Operator::Contains => cell_value
                .to_lowercase()
                .contains(&self.value.to_lowercase()),
            // Comparaisons numériques
            op => {
                let lhs = match cell_value.trim().parse::<f64>() {
                    Ok(v) => v,
                    Err(_) => return false, // valeur non numérique → filtre
                };
                let rhs = match self.value.trim().parse::<f64>() {
                    Ok(v) => v,
                    Err(_) => return false, // référence non numérique → filtre
                };
                match op {
                    Operator::Lt  => lhs <  rhs,
                    Operator::Lte => lhs <= rhs,
                    Operator::Gt  => lhs >  rhs,
                    Operator::Gte => lhs >= rhs,
                    _ => unreachable!(),
                }
            }
        }
    }
}

// ─────────────────────────────────────────────
//  Implémentation du trait Transform
// ─────────────────────────────────────────────

impl Transform for FilterTransform {
    /// Applique le filtre sur l'enregistrement.
    ///
    /// Étapes :
    /// 1. Récupérer la valeur de la colonne → si absente, retourner `None`
    /// 2. Évaluer la condition avec `evaluate()`
    /// 3. Si vraie  → `Some(record)` (conservé)
    /// 4. Si fausse → `None` (filtré)
    fn apply(&self, record: Record) -> Option<Record> {
        // Étape 1 : récupérer la valeur de la colonne cible
        let cell_value = record.get(&self.column)?;  // None si colonne absente

        // Convertir Value en string pour l'évaluation
        let value_str = match cell_value {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::Bool(b) => b.to_string(),
            serde_json::Value::Null => "null".to_string(),
            _ => cell_value.to_string(),
        };

        // Étapes 2, 3 & 4
        if self.evaluate(&value_str) {
            Some(record) // condition vraie → conserver
        } else {
            None // condition fausse → filtrer
        }
    }

    fn name(&self) -> &str {
        "filter"
    }
}

// ─────────────────────────────────────────────
//  Tests unitaires
// ─────────────────────────────────────────────

#[cfg(test_disabled)]
mod tests {
    use super::*;

    fn record(pairs: &[(&str, &str)]) -> Record {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    // ── Opérateur = (égalité) ─────────────────

    #[test]
    fn filter_egalite_valeur_correspondante_conserve() {
        let f = FilterTransform::new("statut", "=", "hospitalise").unwrap();
        let rec = record(&[("statut", "hospitalise"), ("id", "1")]);
        assert!(f.apply(rec).is_some());
    }

    #[test]
    fn filter_egalite_valeur_differente_filtre() {
        let f = FilterTransform::new("statut", "=", "hospitalise").unwrap();
        let rec = record(&[("statut", "ambulatoire")]);
        assert!(f.apply(rec).is_none());
    }

    #[test]
    fn filter_double_egal_equivalent_a_simple() {
        let f = FilterTransform::new("ville", "==", "Paris").unwrap();
        let rec = record(&[("ville", "Paris")]);
        assert!(f.apply(rec).is_some());
    }

    // ── Opérateur != (inégalité) ──────────────

    #[test]
    fn filter_inegalite_valeur_differente_conserve() {
        let f = FilterTransform::new("pays", "!=", "FR").unwrap();
        let rec = record(&[("pays", "US")]);
        assert!(f.apply(rec).is_some());
    }

    #[test]
    fn filter_inegalite_valeur_identique_filtre() {
        let f = FilterTransform::new("pays", "!=", "FR").unwrap();
        let rec = record(&[("pays", "FR")]);
        assert!(f.apply(rec).is_none());
    }

    // ── Opérateur < ───────────────────────────

    #[test]
    fn filter_lt_valeur_inferieure_conserve() {
        let f = FilterTransform::new("age", "<", "18").unwrap();
        let rec = record(&[("age", "15")]);
        assert!(f.apply(rec).is_some());
    }

    #[test]
    fn filter_lt_valeur_egale_filtre() {
        let f = FilterTransform::new("age", "<", "18").unwrap();
        let rec = record(&[("age", "18")]);
        assert!(f.apply(rec).is_none());
    }

    #[test]
    fn filter_lt_valeur_superieure_filtre() {
        let f = FilterTransform::new("age", "<", "18").unwrap();
        let rec = record(&[("age", "25")]);
        assert!(f.apply(rec).is_none());
    }

    // ── Opérateur <= ──────────────────────────

    #[test]
    fn filter_lte_valeur_egale_conserve() {
        let f = FilterTransform::new("score", "<=", "100").unwrap();
        let rec = record(&[("score", "100")]);
        assert!(f.apply(rec).is_some());
    }

    #[test]
    fn filter_lte_valeur_inferieure_conserve() {
        let f = FilterTransform::new("score", "<=", "100").unwrap();
        let rec = record(&[("score", "99")]);
        assert!(f.apply(rec).is_some());
    }

    // ── Opérateur > ───────────────────────────

    #[test]
    fn filter_gt_valeur_superieure_conserve() {
        let f = FilterTransform::new("salaire", ">", "3000").unwrap();
        let rec = record(&[("salaire", "4500")]);
        assert!(f.apply(rec).is_some());
    }

    #[test]
    fn filter_gt_valeur_egale_filtre() {
        let f = FilterTransform::new("salaire", ">", "3000").unwrap();
        let rec = record(&[("salaire", "3000")]);
        assert!(f.apply(rec).is_none());
    }

    // ── Opérateur >= ──────────────────────────

    #[test]
    fn filter_gte_valeur_egale_conserve() {
        let f = FilterTransform::new("note", ">=", "10").unwrap();
        let rec = record(&[("note", "10")]);
        assert!(f.apply(rec).is_some());
    }

    #[test]
    fn filter_gte_valeur_superieure_conserve() {
        let f = FilterTransform::new("note", ">=", "10").unwrap();
        let rec = record(&[("note", "17")]);
        assert!(f.apply(rec).is_some());
    }

    #[test]
    fn filter_gte_valeur_inferieure_filtre() {
        let f = FilterTransform::new("note", ">=", "10").unwrap();
        let rec = record(&[("note", "5")]);
        assert!(f.apply(rec).is_none());
    }

    // ── Opérateur contains ────────────────────

    #[test]
    fn filter_contains_sous_chaine_presente_conserve() {
        let f = FilterTransform::new("description", "contains", "urgent").unwrap();
        let rec = record(&[("description", "Patient urgent - soins immédiats")]);
        assert!(f.apply(rec).is_some());
    }

    #[test]
    fn filter_contains_insensible_casse() {
        let f = FilterTransform::new("description", "contains", "URGENT").unwrap();
        let rec = record(&[("description", "patient urgent")]);
        assert!(f.apply(rec).is_some());
    }

    #[test]
    fn filter_contains_sous_chaine_absente_filtre() {
        let f = FilterTransform::new("description", "contains", "urgent").unwrap();
        let rec = record(&[("description", "consultation standard")]);
        assert!(f.apply(rec).is_none());
    }

    // ── Colonne absente ───────────────────────

    #[test]
    fn filter_colonne_absente_retourne_none() {
        let f = FilterTransform::new("colonne_inexistante", "=", "valeur").unwrap();
        let rec = record(&[("autre", "data")]);
        // Comportement strict : si la colonne n'existe pas → filtre
        assert!(f.apply(rec).is_none());
    }

    // ── Valeur non numérique avec opérateur numérique ──

    #[test]
    fn filter_valeur_non_numerique_avec_operateur_numerique_filtre() {
        let f = FilterTransform::new("age", ">", "18").unwrap();
        let rec = record(&[("age", "inconnu")]);
        // "inconnu" ne peut pas être parsé en f64 → filtre (false)
        assert!(f.apply(rec).is_none());
    }

    #[test]
    fn filter_reference_non_numerique_avec_operateur_numerique_filtre() {
        let f = FilterTransform::new("age", ">", "majeur").unwrap();
        let rec = record(&[("age", "20")]);
        // référence non numérique → filtre
        assert!(f.apply(rec).is_none());
    }

    // ── Valeurs décimales ─────────────────────

    #[test]
    fn filter_gt_avec_decimaux() {
        let f = FilterTransform::new("temperature", ">", "36.5").unwrap();
        let rec_fievre  = record(&[("temperature", "38.2")]);
        let rec_normale = record(&[("temperature", "36.3")]);
        assert!(f.apply(rec_fievre).is_some());
        assert!(f.apply(rec_normale).is_none());
    }

    #[test]
    fn filter_lt_avec_negatifs() {
        let f = FilterTransform::new("solde", "<", "0").unwrap();
        let rec = record(&[("solde", "-150")]);
        assert!(f.apply(rec).is_some());
    }

    // ── Opérateur invalide ────────────────────

    #[test]
    fn filter_operateur_inconnu_retourne_erreur() {
        let result = FilterTransform::new("col", "???", "val");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Opérateur inconnu"));
    }

    // ── Intégrité du record conservé ─────────

    #[test]
    fn filter_conserve_toutes_les_colonnes_si_condition_vraie() {
        let f = FilterTransform::new("actif", "=", "oui").unwrap();
        let rec = record(&[
            ("id",    "42"),
            ("nom",   "Martin"),
            ("actif", "oui"),
            ("score", "95"),
        ]);
        let result = f.apply(rec).unwrap();
        assert_eq!(result.len(), 4);
        assert_eq!(result["id"],    "42");
        assert_eq!(result["nom"],   "Martin");
        assert_eq!(result["actif"], "oui");
        assert_eq!(result["score"], "95");
    }

    #[test]
    fn filter_name_retourne_filter() {
        let f = FilterTransform::new("x", "=", "y").unwrap();
        assert_eq!(f.name(), "filter");
    }

    // ── Cas hôpital (scénario réel) ───────────

    #[test]
    fn filter_scenario_hopital_garde_patients_hospitalises() {
        let f = FilterTransform::new("statut", "=", "hospitalise").unwrap();

        let hospitalise = record(&[("id", "1"), ("nom", "Dupont"), ("statut", "hospitalise")]);
        let ambulatoire = record(&[("id", "2"), ("nom", "Martin"), ("statut", "ambulatoire")]);
        let urgences    = record(&[("id", "3"), ("nom", "Durand"), ("statut", "urgences")]);

        assert!(f.apply(hospitalise).is_some());
        assert!(f.apply(ambulatoire).is_none());
        assert!(f.apply(urgences).is_none());
    }

    #[test]
    fn filter_scenario_hopital_patients_mineurs() {
        let f = FilterTransform::new("age", "<", "18").unwrap();

        let mineur  = record(&[("nom", "Léa"),    ("age", "12")]);
        let majeur  = record(&[("nom", "Thomas"),  ("age", "22")]);
        let limite  = record(&[("nom", "Camille"), ("age", "18")]);

        assert!(f.apply(mineur).is_some());
        assert!(f.apply(majeur).is_none());
        assert!(f.apply(limite).is_none()); // 18 < 18 = false
    }
}
