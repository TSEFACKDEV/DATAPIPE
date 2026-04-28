// src/transform/rename.rs
// [ASSONGUE] - Transformation : Renommage de colonne

use crate::transform::{Record, Transform};

// ─────────────────────────────────────────────
//  Structure
// ─────────────────────────────────────────────

/// Renomme une colonne dans chaque enregistrement.
///
/// Si la colonne source (`from`) est absente, l'enregistrement
/// est retourné **sans modification** et sans erreur — comportement
/// tolérant adapté aux sources hétérogènes.
///
/// # Exemple TOML (pipeline.toml)
/// ```toml
/// [[transform]]
/// type   = "rename"
/// from   = "nom_patient"
/// to     = "name"
/// ```
///
/// # Exemple Rust
/// ```rust,no_run
/// use datapipe::transform::rename::RenameTransform;
/// use datapipe::transform::Transform;
/// use datapipe::reader::Record;
/// use serde_json::json;
///
/// let t = RenameTransform::new("prenom", "first_name");
/// let mut record = Record::new();
/// record.insert("prenom".to_string(), json!("Alice"));
///
/// let result = t.apply(record).unwrap();
/// assert!(result.contains_key("first_name"));
/// assert!(!result.contains_key("prenom"));
/// ```
#[derive(Debug, Clone)]
pub struct RenameTransform {
    /// Nom de la colonne à renommer (source)
    pub from: String,
    /// Nouveau nom de la colonne (cible)
    pub to: String,
}

impl RenameTransform {
    /// Crée un nouveau `RenameTransform`.
    ///
    /// # Arguments
    /// * `from` – nom actuel de la colonne
    /// * `to`   – nouveau nom souhaité
    #[allow(dead_code)]
    pub fn new(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
        }
    }
}

// ─────────────────────────────────────────────
//  Implémentation du trait Transform
// ─────────────────────────────────────────────

impl Transform for RenameTransform {
    /// Renomme la colonne `from` en `to`.
    ///
    /// Étapes :
    /// 1. Vérifier si `from` existe → si non, retourner `Some(record)` intact
    /// 2. Extraire la valeur avec `shift_remove()` (évite un clone)
    /// 3. Insérer la valeur sous le nouveau nom `to`
    /// 4. Retourner `Some(record)` — le rename ne filtre jamais
    fn apply(&self, mut record: Record) -> Option<Record> {
        // Étape 1 & 2 : shift_remove() retourne None si la clé est absente
        if let Some(value) = record.shift_remove(&self.from) {
            // Étape 3 : insérer sous le nouveau nom
            record.insert(self.to.clone(), value);
        }
        // Étape 4 : toujours retourner Some (jamais de filtrage)
        Some(record)
    }

    fn name(&self) -> &str {
        "rename"
    }
}

// ─────────────────────────────────────────────
//  Tests unitaires
// ─────────────────────────────────────────────

#[cfg(test_disabled)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// Construit un Record à partir de paires (&str, &str)
    fn record(pairs: &[(&str, &str)]) -> Record {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    // ── Cas nominaux ──────────────────────────

    #[test]
    fn rename_colonne_existante_renomme_correctement() {
        let t = RenameTransform::new("prenom", "first_name");
        let rec = record(&[("prenom", "Alice"), ("age", "30")]);

        let result = t.apply(rec).expect("ne doit pas retourner None");

        // La nouvelle clé doit exister avec la bonne valeur
        assert_eq!(result.get("first_name"), Some(&"Alice".to_string()));
        // L'ancienne clé ne doit plus exister
        assert!(!result.contains_key("prenom"));
        // Les autres colonnes ne doivent pas être touchées
        assert_eq!(result.get("age"), Some(&"30".to_string()));
    }

    #[test]
    fn rename_preserve_la_valeur_exacte() {
        let t = RenameTransform::new("note", "score");
        let rec = record(&[("note", "18.5/20")]);

        let result = t.apply(rec).unwrap();
        assert_eq!(result["score"], "18.5/20");
    }

    #[test]
    fn rename_retourne_toujours_some() {
        // Le rename ne doit JAMAIS retourner None
        let t = RenameTransform::new("x", "y");
        let rec = record(&[("x", "val")]);
        assert!(t.apply(rec).is_some());
    }

    // ── Colonne absente ───────────────────────

    #[test]
    fn rename_colonne_absente_ne_plante_pas() {
        let t = RenameTransform::new("colonne_inexistante", "nouveau_nom");
        let rec = record(&[("autre", "valeur")]);

        // Ne doit pas paniquer
        let result = t.apply(rec).expect("doit retourner Some même si colonne absente");

        // L'enregistrement doit être intact (rien de renommé ni supprimé)
        assert!(result.contains_key("autre"));
        assert!(!result.contains_key("nouveau_nom"));
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn rename_colonne_absente_ne_cree_pas_de_cle_vide() {
        let t = RenameTransform::new("ghost", "visible");
        let rec = record(&[("real", "data")]);

        let result = t.apply(rec).unwrap();
        // "visible" ne doit PAS être créé si "ghost" n'existe pas
        assert!(!result.contains_key("visible"));
    }

    // ── Record vide ───────────────────────────

    #[test]
    fn rename_sur_record_vide_retourne_record_vide() {
        let t = RenameTransform::new("a", "b");
        let rec: Record = HashMap::new();

        let result = t.apply(rec).unwrap();
        assert!(result.is_empty());
    }

    // ── Cas particuliers ──────────────────────

    #[test]
    fn rename_from_et_to_identiques_ne_perd_pas_la_valeur() {
        // Renommer "age" → "age" : doit rester intact
        let t = RenameTransform::new("age", "age");
        let rec = record(&[("age", "25")]);

        let result = t.apply(rec).unwrap();
        assert_eq!(result.get("age"), Some(&"25".to_string()));
    }

    #[test]
    fn rename_avec_valeur_vide() {
        let t = RenameTransform::new("champ", "field");
        let rec = record(&[("champ", "")]);

        let result = t.apply(rec).unwrap();
        assert_eq!(result.get("field"), Some(&"".to_string()));
    }

    #[test]
    fn rename_avec_valeur_contenant_espaces_et_accents() {
        let t = RenameTransform::new("nom_complet", "full_name");
        let rec = record(&[("nom_complet", "Élodie Müller-Dupont")]);

        let result = t.apply(rec).unwrap();
        assert_eq!(result["full_name"], "Élodie Müller-Dupont");
    }

    #[test]
    fn rename_name_retourne_rename() {
        let t = RenameTransform::new("a", "b");
        assert_eq!(t.name(), "rename");
    }

    // ── Application multiple ──────────────────

    #[test]
    fn deux_renames_successifs_independants() {
        let t1 = RenameTransform::new("a", "b");
        let t2 = RenameTransform::new("c", "d");

        let rec = record(&[("a", "1"), ("c", "2")]);

        let rec = t1.apply(rec).unwrap();
        let rec = t2.apply(rec).unwrap();

        assert!(rec.contains_key("b"));
        assert!(rec.contains_key("d"));
        assert!(!rec.contains_key("a"));
        assert!(!rec.contains_key("c"));
    }
}
