# DataPipe - Outil ETL en Rust

DataPipe est un outil **ETL** (Extract, Transform, Load) haute-performance développé en Rust pour traiter et transformer des données en batch.

## 🎯 Objectif

Lire un fichier de données (CSV, JSON, texte délimité), appliquer des transformations (renommage, filtrage, calcul), et écrire le résultat dans un autre format.

**Exemple :**
```
CSV → [Rename: "nom_complet" → "nom"] 
    → [Filter: departement = "IT"]
    → [Drop: "mot_de_passe"]
    → JSON
```

---

## 📦 Installation

### Prérequis
- **Rust 1.75.0** ou supérieur ([installer](https://rustup.rs))
- **Cargo** (inclus avec Rust)

### Compilation
```bash
git clone <repo>
cd datapipe
cargo build --release
```

L'exécutable se trouvera dans `target/release/datapipe`

---

## 🚀 Utilisation

### 1. Créer un fichier de configuration `pipeline.toml`

```toml
# Source de données
[source]
format = "csv"              # Format: csv, json, delimited
path = "data/input.csv"
delimiter = ","             # Pour CSV et délimité

# Destination
[destination]
format = "json"             # Format: csv, json, jsonl
path = "output/result.json"

# Transformations (optionnel, dans l'ordre d'exécution)
[[transforms]]
type = "rename"
from = "nom_complet"
to = "nom"

[[transforms]]
type = "filter"
column = "departement"
value = "Informatique"
operator = "="

[[transforms]]
type = "drop"
column = "mot_de_passe"

[[transforms]]
type = "cast"
column = "salaire"
target_type = "number"

[[transforms]]
type = "compute"
new_column = "prime"
expression = "salaire * 0.1"
```

### 2. Exécuter le pipeline

#### Mode normal
```bash
./target/release/datapipe --config pipeline.toml
```

#### Mode dry-run (simulation sans écriture)
```bash
./target/release/datapipe --config pipeline.toml --dry-run
```

#### Mode watch (relancer à chaque modification de source)
```bash
./target/release/datapipe --config pipeline.toml --watch --interval 30
```

---

## 📋 Formats supportés

### Lecture (Source)
- **CSV** : Fichiers séparés par virgule/point-virgule/tabulation
- **JSON** : Tableaux d'objets `[{}, {}]`
- **Délimité** : Texte avec délimiteur personnalisé

### Écriture (Destination)
- **CSV** : Avec en-têtes
- **JSON** : Tableau d'objets
- **JSONL** : Une ligne JSON par record

---

## 🔄 Transformations disponibles

| Type | Paramètres | Exemple |
|------|-----------|---------|
| `rename` | `from`, `to` | Renommer "nom_complet" → "nom" |
| `filter` | `column`, `value`, `operator` | Garder si `dept = "IT"` |
| `drop` | `column` | Supprimer la colonne "pwd" |
| `cast` | `column`, `target_type` | Convertir "123" (string) → 123 (number) |
| `compute` | `new_column`, `expression` | Ajouter `prime = salaire * 0.1` |

---

## 🏗️ Architecture

```
datapipe/
├── src/
│   ├── main.rs              # Point d'entrée CLI (TSEFACK)
│   ├── config.rs            # Configuration TOML (TSEFACK)
│   ├── pipeline.rs          # Orchestrateur (TSEFACK)
│   ├── stats.rs             # Rapport d'exécution
│   │
│   ├── reader/              # Lecteurs de formats
│   │   ├── csv_reader.rs    # CSV (NZEUTEM)
│   │   ├── json_reader.rs   # JSON (DIOM)
│   │   └── delimited_reader.rs # Texte (DIOM)
│   │
│   ├── transform/           # Transformations
│   │   ├── rename.rs        # Rename (ASSONGUE)
│   │   ├── filter.rs        # Filter (ASSONGUE)
│   │   ├── cast.rs          # Cast (NOLACK)
│   │   ├── compute.rs       # Compute (NOLACK)
│   │   ├── drop.rs          # Drop (NOLACK)
│   │   └── factory.rs       # Fabrique (NOLACK)
│   │
│   └── writer/              # Écrivains de formats
│       ├── csv_writer.rs    # CSV (NGLITANG)
│       ├── json_writer.rs   # JSON (NGLITANG)
│       ├── jsonl_writer.rs  # JSONL (NGANSOP)
│       └── factory.rs       # Fabrique (NGANSOP)
│
├── examples/                # Exemples de pipelines
├── data/                    # Données de test
├── pipeline.toml            # Configuration d'exemple
└── Cargo.toml               # Dépendances
```

---

## 📊 Exemple complet

### Données d'entrée (`data/employees.csv`)
```csv
nom_complet,departement,salaire,mot_de_passe
Alice Martin,Informatique,5000,secret123
Bob Dupont,RH,4000,pass456
Charlie Bernard,Informatique,5500,pwd789
```

### Configuration (`pipeline.toml`)
```toml
[source]
format = "csv"
path = "data/employees.csv"

[destination]
format = "json"
path = "output/employees.json"

[[transforms]]
type = "rename"
from = "nom_complet"
to = "nom"

[[transforms]]
type = "filter"
column = "departement"
value = "Informatique"
operator = "="

[[transforms]]
type = "drop"
column = "mot_de_passe"
```

### Résultat (`output/employees.json`)
```json
[
  {
    "nom": "Alice Martin",
    "departement": "Informatique",
    "salaire": "5000"
  },
  {
    "nom": "Charlie Bernard",
    "departement": "Informatique",
    "salaire": "5500"
  }
]
```

### Rapport d'exécution
```
Records lus : 3
Records transformés : 2
Records filtrés : 1
Records écrits : 2
Erreurs : 0
Durée : 12ms
```

---

## 🧪 Tests

```bash
# Tests unitaires
cargo test

# Tests avec valgrind (fuites mémoire)
cargo build --release
valgrind ./target/release/datapipe --config pipeline.toml

# Benchmark
cargo build --release
time ./target/release/datapipe --config pipeline.toml
```

---

## 👥 Équipe Groupe 6

| Rôle | Nom | Module |
|------|-----|--------|
| Chef Projet | TSEFACK Calvin Klein | main.rs, config.rs, pipeline.rs |
| CSV Reader | NZEUTEM Eunice | csv_reader.rs |
| JSON/Délimité | DIOM Lucraine | json_reader.rs, delimited_reader.rs |
| Rename/Filter | ASSONGUE Muriel | rename.rs, filter.rs |
| Cast/Compute/Drop | NOLACK Kawunjibi | cast.rs, compute.rs, drop.rs, factory.rs |
| CSV/JSON Writer | NGLITANG | csv_writer.rs, json_writer.rs |
| JSONL Writer | NGANSOP | jsonl_writer.rs, factory.rs |
| Stats/Rapport | DONFACK | stats.rs, report.rs |
| Join | ATEKOUMBO | join.rs |
| Watch/Dry-run | NJOH | watch.rs |

---

## 📝 Conventions de code

- **Nommage** : `snake_case` pour fonctions/variables, `PascalCase` pour types
- **Comments** : En français pour la lisibilité en équipe
- **Errors** : Utiliser `anyhow::Result` + contexte détaillé
- **Imports** : Ordonner par: std lib → external crates → internal modules

---

## 🐛 Dépannage

### "Format source non supporté"
**Cause** : La source.format n'est pas "csv", "json", ou "delimited"
**Solution** : Vérifier pipeline.toml

### "Fichier introuvable"
**Cause** : Le chemin source n'existe pas
**Solution** : Vérifier le chemin relatif depuis le répertoire d'exécution

### "Délimiteur requis pour format 'delimited'"
**Cause** : Format délimité sans délimiteur spécifié
**Solution** : Ajouter `delimiter = ";"` ou `delimiter = "\t"`

---

## 📄 Licence

ENSP Yaoundé - Projet Programmation Système 2024-2025


