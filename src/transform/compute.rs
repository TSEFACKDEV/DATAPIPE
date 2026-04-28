// =============================================================================
// FICHIER : src/transform/compute.rs
// AUTEUR  : NOLACK KAWUNJIBI FRANGE PARKER
// RÔLE    : Transformation de CALCUL — crée une nouvelle colonne dont la valeur
//           est calculée à partir d'autres colonnes du record.
// =============================================================================
//
// ROLE DU FICHIER
// Dans les données métier, on a souvent besoin de CALCULER de nouvelles colonnes.
// Exemples concrets :
//   - prime = salaire * 0.1                  (10% du salaire)
//   - total_ttc = montant_ht * 1.1925        (TVA camerounaise 19.25%)
//   - nom_complet = prenom + " " + nom       (concaténation)
//   - rendement = production / superficie    (ONG agricole)
//
// FORMAT DE L'EXPRESSION (syntaxe simplifiée) :
//   "colonne1 opérateur colonne_ou_constante"
//
//   Opérateurs numériques : *, +, -, /
//   Opérateur texte       : concat  (ex: "prenom concat nom")
//
//   Exemples :
//     "prix + frais"          -> total = prix + frais
//     "montant / 655.957"     -> conversion XAF vers EUR
//     "prenom concat nom"     -> nom_complet
//
// EXEMPLE D'UTILISATION dans pipeline.toml :
//   [[transforms]]
//   type = "compute"
//   new_column = "prime"
//   expression = "salaire * 0.1"
// =============================================================================

use super::{Record, Transform};
use serde_json::{json, Value};

// =============================================================================
// STRUCTURE ComputeTransform
// =============================================================================
pub struct ComputeTransform {
    /// Nom de la nouvelle colonne à créer
    pub new_column: String,

    /// Expression de calcul sous forme de string (ex: "salaire * 0.1")
    /// Format : "operande_gauche opérateur operande_droite"
    pub expression: String,
}

// =============================================================================
// IMPLÉMENTATION DU TRAIT Transform POUR ComputeTransform
// =============================================================================
impl Transform for ComputeTransform {
    fn apply(&self, mut record: Record) -> Option<Record> {

        // On délègue le calcul à la fonction parse_and_evaluate.
        // `match` condition pour gérer le résultat (succès ou erreur).
        match parse_and_evaluate(&self.expression, &record) {

            // Calcul réussi -> on insère la nouvelle colonne dans le record
            Ok(result) => {
                // `self.new_column.clone()` : on duplique le nom de colonne
                // car insert() prend ownership de la clé.
                record.insert(self.new_column.clone(), result);
            }

            // Calcul échoué -> on avertit.
            // Le record continue dans le pipeline SANS la nouvelle colonne.
            Err(e) => {
                eprintln!(
                    "[WARN] ComputeTransform: erreur dans expression '{}' → {}",
                    self.expression, e
                );
            }
        }

        // On retourne toujours Some(record).
        Some(record)
    }

    fn name(&self) -> &str {
        "compute"
    }
}

// =============================================================================
// FONCTION PRIVÉE : parse_and_evaluate
// =============================================================================
// Elle parse l'expression et calcule.
//
// PARAMÈTRES :
//   expression : la string de l'expression
//   record     : référence vers le record courant (pour lire les colonnes)
//
// RETOURNE : Result<Value, String>
//   Ok(Value)   -> le résultat du calcul sous forme de valeur JSON
//   Err(String) -> message d'erreur si quelque chose s'est mal passé
//
// Elle est privée -> utilisable uniquement dans ce fichier.
fn parse_and_evaluate(expression: &str, record: &Record) -> Result<Value, String> {

    // `split_whitespace()` divise la string sur les espaces/tabulations/retours.
    // `.collect::<Vec<_>>()` transforme l'itérateur en vecteur (liste).
    // `Vec<_>` : Vec de quelque chose (Rust infère le type automatiquement).
    let parts: Vec<&str> = expression.split_whitespace().collect();

    // On vérifie qu'on a bien exactement 3 éléments : gauche, opérateur, droite.
    if parts.len() != 3 {
        return Err(format!(
            "Expression invalide '{}': attendu 'gauche opérateur droite' (3 éléments, got {})",
            expression, parts.len()
        ));
    }

    // On destructure le vecteur en 3 variables.
    // `parts[0]`, `parts[1]`, `parts[2]` sont des &str (références vers des tranches de string).
    let left_elt = parts[0];   // Gauche  : colonne ou constante
    let operator   = parts[1];   // Opérateur
    let right_elt = parts[2];  // Droite : colonne ou constante

    // Cas spécial : opérateur "concat" pour la concaténation de texte
    if operator == "concat" {
        return evaluate_concat(left_elt, right_elt, record);
    }

    let left_num  = resolve_number(left_elt, record)?;  // `?` = propage l'erreur
    let right_num = resolve_number(right_elt, record)?;

    // `?` est l'opérateur de propagation d'erreur en Rust.
    // Si resolve_number retourne Err(e), la fonction courante retourne immédiatement Err(e).

    // On calcule selon l'opérateur
    let result = match operator {
        // Multiplication : prix * tva, salaire * coefficient, etc.
        "*" => left_num * right_num,

        // Addition : montant + frais, score + bonus, etc.
        "+" => left_num + right_num,

        // Soustraction : brut - retenues, total - remise, etc.
        "-" => left_num - right_num,

        // Division : total / nombre, production / superficie, etc.
        "/" => {
            // ATTENTION : division par zéro !
            // En Rust, diviser un f64 par 0.0 donne f64::INFINITY (infini) qui est une donnée invalide.
            if right_num == 0.0 {
                return Err(format!("Division par zéro dans '{}'", expression));
            }
            left_num / right_num
        }

        // Opérateur inconnu -> erreur explicite
        op => {
            return Err(format!(
                "Opérateur inconnu '{}' dans '{}'. Opérateurs supportés: *, +, -, /, concat",
                op, expression
            ));
        }
    };

    // `json!(result)` : on emballe le f64 dans un Value::Number pour le retourner.
    Ok(json!(result))
}

// =============================================================================
// FONCTION PRIVÉE : resolve_number
// =============================================================================
// Résout un token en valeur numérique (f64).
// Un token peut être :
//   1. Un NOMBRE LITTÉRAL : "0.1", "1.1925", "655.957"
//   2. Un NOM DE COLONNE : "salaire", "montant", "production"
//
// RETOURNE : Result<f64, String>
fn resolve_number(token: &str, record: &Record) -> Result<f64, String> {

    // `.parse::<f64>()` tente la conversion directe en nombre
    if let Ok(num) = token.parse::<f64>() {
        // C'est un nombre littéral -> on le retourne directement.
        return Ok(num);
    }

    // si Ce n'est pas un nombre littéral -> c'est un nom de colonne.
    // On cherche la valeur de cette colonne dans le record.
    match record.get(token) {

        // La colonne existe et c'est un Number JSON.
        // `.as_f64()` extrait le f64 depuis le Number JSON.
        Some(Value::Number(n)) => n.as_f64().ok_or_else(|| {
            // `ok_or_else` convertit Option en Result.
            // Si as_f64() retourne None → on crée une Err.
            format!("Impossible d'extraire f64 depuis le nombre dans la colonne '{}'", token)
        }),

        // La colonne existe et c'est une String → on tente de la parser en nombre.
        Some(Value::String(s)) => s.trim().parse::<f64>().map_err(|_| {
            // `map_err` transforme l'erreur de parsing en notre message personnalisé.
            format!(
                "La colonne '{}' contient '{}' qui n'est pas un nombre valide",
                token, s
            )
        }),

        // La colonne existe mais c'est un type non numérique (Bool, Array, Object, Null)
        Some(other) => Err(format!(
            "La colonne '{}' contient un type non numérique : {:?}",
            token, other
        )),

        // La colonne n'existe pas du tout dans le record
        None => Err(format!(
            "Colonne '{}' introuvable dans le record (disponibles: {:?})",
            token,
            record.keys().collect::<Vec<_>>()
        )),
    }
}

// =============================================================================
// FONCTION PRIVÉE : evaluate_concat
// =============================================================================
// Concatène deux valeurs textuelles avec un espace entre elles.
// Ex: "prenom concat nom" → "Pierre Mbodji"
fn evaluate_concat(left: &str, right: &str, record: &Record) -> Result<Value, String> {
    
    // Résout le token gauche en String
    let left_str  = resolve_string(left, record)?;
    let right_str = resolve_string(right, record)?;

    // Concaténation avec un espace séparateur (comportement par défaut)
    // `format!` est le printf/sprintf de Rust.
    Ok(json!(format!("{} {}", left_str, right_str)))
}

// =============================================================================
// FONCTION PRIVÉE : resolve_string
// =============================================================================
// Résout un token en valeur String.
// Peut être un littéral (entre guillemets dans l'expression) ou un nom de colonne.
fn resolve_string(token: &str, record: &Record) -> Result<String, String> {

    // Essai 1 : est-ce un littéral entre guillemets ?
    // Ex: dans l'expression `prenom concat " "` le token `" "` est un littéral.
    // Note: dans une expression TOML, les guillemets devront etre échappés.
    if token.starts_with('"') && token.ends_with('"') && token.len() >= 2 {
        // On extrait le contenu entre les guillemets.
        // `&token[1..token.len()-1]` = slice de la position 1 à l'avant-dernière pour ne pas compter les guillements
        return Ok(token[1..token.len()-1].to_string());
    }

    // Essai 2 : c'est un nom de colonne -> on cherche dans le record.
    match record.get(token) {

        // String JSON -> on clone le contenu
        Some(Value::String(s)) => Ok(s.clone()),

        // Number, Bool -> on convertit en string via to_string()
        Some(Value::Number(n)) => Ok(n.to_string()),
        Some(Value::Bool(b))   => Ok(b.to_string()),

        // Null -> string vide
        Some(Value::Null)      => Ok(String::new()),

        // Objet/Tableau complexe -> représentation JSON
        Some(other)            => Ok(other.to_string()),

        // Colonne inexistante -> erreur
        None => Err(format!("Colonne '{}' introuvable pour concat", token)),
    }
}

// =============================================================================
// TESTS UNITAIRES
// =============================================================================
#[cfg(test_disabled)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::HashMap;

    /// Crée un Record de test avec plusieurs colonnes numériques.
    fn make_record_num(paires: &[(&str, f64)]) -> Record {
        let mut record = HashMap::new();
        for (k, v) in paires {
            // On insère chaque paire clé/valeur numérique
            record.insert(k.to_string(), json!(v));
        }
        record
    }

    /// Crée un Record avec des colonnes textuelles.
    fn make_record_str(paires: &[(&str, &str)]) -> Record {
        let mut record = HashMap::new();
        for (k, v) in paires {
            record.insert(k.to_string(), json!(v));
        }
        record
    }

    #[test]
    fn test_multiplication_salaire_prime() {
        // prime = salaire * 0.1  (10% du salaire)
        let transform = ComputeTransform {
            new_column: "prime".to_string(),
            expression: "salaire * 0.1".to_string(),
        };
        let record = make_record_num(&[("salaire", 150000.0)]);
        let result = transform.apply(record).unwrap();
        // 150000 * 0.1 = 15000.0
        assert_eq!(result["prime"], json!(15000.0_f64));
    }

    #[test]
    fn test_addition_montant_frais() {
        let transform = ComputeTransform {
            new_column: "total".to_string(),
            expression: "montant + frais".to_string(),
        };
        let record = make_record_num(&[("montant", 100000.0), ("frais", 5000.0)]);
        let result = transform.apply(record).unwrap();
        assert_eq!(result["total"], json!(105000.0_f64));
    }

    #[test]
    fn test_soustraction() {
        let transform = ComputeTransform {
            new_column: "net".to_string(),
            expression: "brut - retenues".to_string(),
        };
        let record = make_record_num(&[("brut", 200000.0), ("retenues", 30000.0)]);
        let result = transform.apply(record).unwrap();
        assert_eq!(result["net"], json!(170000.0_f64));
    }

    #[test]
    fn test_division_rendement() {
        // rendement_hectare = production / superficie
        let transform = ComputeTransform {
            new_column: "rendement".to_string(),
            expression: "production / superficie".to_string(),
        };
        let record = make_record_num(&[("production", 1000.0), ("superficie", 5.0)]);
        let result = transform.apply(record).unwrap();
        assert_eq!(result["rendement"], json!(200.0_f64));
    }

    #[test]
    fn test_division_par_zero_ne_plante_pas() {
        let transform = ComputeTransform {
            new_column: "ratio".to_string(),
            expression: "montant / zero".to_string(),
        };
        let record = make_record_num(&[("montant", 100.0), ("zero", 0.0)]);
        // Ne doit pas paniquer → le record passe sans la nouvelle colonne
        let result = transform.apply(record);
        assert!(result.is_some());
        // La colonne "ratio" ne doit pas avoir été créée
        assert!(result.unwrap().get("ratio").is_none());
    }

    #[test]
    fn test_concat_prenom_nom() {
        let transform = ComputeTransform {
            new_column: "nom_complet".to_string(),
            expression: "prenom concat nom".to_string(),
        };
        let record = make_record_str(&[("prenom", "Jean"), ("nom", "Mbodj")]);
        let result = transform.apply(record).unwrap();
        assert_eq!(result["nom_complet"], json!("Jean Mbodj"));
    }

    #[test]
    fn test_colonne_inexistante_ne_filtre_pas() {
        // Si une colonne référencée n'existe pas → le record passe sans nouvelle colonne
        let transform = ComputeTransform {
            new_column: "resultat".to_string(),
            expression: "inexistant * 2".to_string(),
        };
        let record = make_record_num(&[("autre", 100.0)]);
        let result = transform.apply(record);
        assert!(result.is_some()); // Ne filtre pas !
    }

    #[test]
    fn test_constante_numerique() {
        // On peut utiliser une constante directement dans l'expression
        let transform = ComputeTransform {
            new_column: "tva".to_string(),
            expression: "ht * 1.1925".to_string(), // TVA camerounaise 19.25%
        };
        let record = make_record_num(&[("ht", 10000.0)]);
        let result = transform.apply(record).unwrap();
        // 10000 * 1.1925 = 11925.0
        let tva = result["tva"].as_f64().unwrap();
        // Comparaison avec tolérance car les flottants ne sont pas exactement précis
        assert!((tva - 11925.0).abs() < 0.01, "Attendu ~11925.0, reçu {}", tva);
    }
}
