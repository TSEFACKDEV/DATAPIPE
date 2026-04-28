// On importe serde_json::Value — c'est un type Rust qui peut représenter
// N'IMPORTE QUELLE valeur JSON : texte, nombre, booléen, null, liste, objet.
use serde_json::Value;

// On importe IndexMap au lieu de HashMap classique pour garder l'ORDRE
// des colonnes (important pour l'écriture CSV où l'ordre doit être stable).
use indexmap::IndexMap;

// On utilise IndexMap<String, Value> plutôt que HashMap<String, Value> pour
// que les colonnes restent dans le bon ordre (comme dans le fichier source).
//
pub type Record = IndexMap<String, Value>;

// Box<dyn Iterator<...>> signifie : "un itérateur, quel que soit son type exact"
// C'est nécessaire car CSV, JSON et texte ont chacun leur propre type d'itérateur.
//
pub trait SourceReader {
    fn records(&self) -> Box<dyn Iterator<Item = anyhow::Result<Record>>>;
}

// On déclare les sous-modules pour que Rust sache où chercher les fichiers
pub mod csv_reader;
pub mod json_reader;
pub mod delimited_reader;