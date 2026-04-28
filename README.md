# 🚀 DataPipe - Outil ETL Haute Performance en Rust

## 📋 Table des Matières
1. [Vue d'ensemble](#vue-densemble)
2. [Installation](#installation)
3. [Utilisation](#utilisation)
4. [Formats supportés](#formats-supportés)
5. [Transformations disponibles](#transformations-disponibles)
6. [Architecture](#architecture)
7. [Équipe](#équipe)

---

## 🎯 Vue d'ensemble

**DataPipe** est un outil **ETL** (Extract, Transform, Load) haute-performance développé en Rust pour traiter et transformer des données en batch.

### Cas d'usage
- 📊 Conversion de formats de données (CSV ↔ JSON ↔ JSONL)
- 🔄 Nettoyage et standardisation de données
- 🎯 Filtrage et extraction sélective
- 🧮 Calculs et enrichissements
- 🚀 Traitement de gros volumes sans charger tout en RAM

### Pourquoi Rust ?
- ⚡ **Performance**: Aussi rapide que C/C++
- 🔒 **Sécurité mémoire**: Pas de crashes mystérieux
- 📦 **Compilation**: Exécutable autonome, zéro dépendance runtime
- 🎯 **Streaming**: Traiter des fichiers de 10 Go avec <100 MB de RAM

---

## 💻 Installation

### Prérequis
- Rust 1.75.0+ ([installer](https://rustup.rs))
- Cargo (inclus avec Rust)

### Compilation
```bash
git clone <repo>
cd datapipe
cargo build --release
```

L'exécutable se trouvera dans `target/release/datapipe`

---

## 📖 Utilisation

### 1. Créer un fichier de configuration `pipeline.toml`

```toml
# Source de données
[source]
format = "csv"                    # csv, json, ou delimited
path = "data/employes.csv"
delimiter = ","                   # Pour CSV et délimité

# Destination
[destination]
format = "json"                   # csv, json, ou jsonl
path = "output/employes.json"

# Transformations (optionnel, appliquées en ordre)
[[transforms]]
type = "rename"
from = "nom_complet"
to = "nom"

[[transforms]]
type = "filter"
column = "departement"
value = "Informatique"
operator = "="                    # =, !=, <, <=, >, >=, contains

[[transforms]]
type = "drop"
column = "mot_de_passe"

[[transforms]]
type = "cast"
column = "salaire"
target_type = "number"            # string, number, boolean

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

#### Mode dry-run (aperçu sans écriture)
```bash
./target/release/datapipe --config pipeline.toml --dry-run
```

#### Mode watch (relancer à chaque modification)
```bash
./target/release/datapipe --config pipeline.toml --watch --interval 30
```

---

## 📦 Formats Supportés

### Lecture (Source)
| Format | Extension | Exemple |
|--------|-----------|---------|
| CSV | `.csv` | Fichiers Excel exportés |
| JSON | `.json` | Tableaux d'objets `[{...}]` |
| Délimité | `.txt` | Texte avec délimiteur custom |

### Écriture (Destination)
| Format | Extension | Avantage |
|--------|-----------|----------|
| CSV | `.csv` | Compatible Excel |
| JSON | `.json` | Lisible, un tableau |
| JSONL | `.jsonl` | **Recommandé** - streaming pur |

---

## 🔄 Transformations Disponibles

### 1. rename - Renommage de colonne
```toml
[[transforms]]
type = "rename"
from = "nom_client"
to = "customer_name"
```

### 2. filter - Filtrage par condition
```toml
[[transforms]]
type = "filter"
column = "statut"
value = "actif"
operator = "="  # ou !=, <, <=, >, >=, contains
```

### 3. drop - Suppression de colonne
```toml
[[transforms]]
type = "drop"
column = "mot_de_passe"
```

### 4. cast - Conversion de type
```toml
[[transforms]]
type = "cast"
column = "age"
target_type = "number"  # string, number, boolean
```

### 5. compute - Calcul de nouvelles colonnes
```toml
[[transforms]]
type = "compute"
new_column = "salaire_annuel"
expression = "salaire * 12"
```

---

## 🏗️ Architecture

```
datapipe/
├── src/
│   ├── main.rs              # CLI
│   ├── config.rs            # Configuration TOML
│   ├── pipeline.rs          # Orchestrateur
│   ├── stats.rs             # Statistiques
│   │
│   ├── reader/              # Lecteurs
│   │   ├── mod.rs
│   │   ├── csv_reader.rs
│   │   ├── json_reader.rs
│   │   └── delimited_reader.rs
│   │
│   ├── transform/           # Transformations
│   │   ├── mod.rs
│   │   ├── rename.rs
│   │   ├── filter.rs
│   │   ├── cast.rs
│   │   ├── compute.rs
│   │   ├── drop.rs
│   │   └── factory.rs
│   │
│   ├── writer/              # Écrivains
│   │   ├── mod.rs
│   │   ├── csv_writer.rs
│   │   ├── json_writer.rs
│   │   ├── jsonl_writer.rs
│   │   └── factory.rs
│   │
│   ├── validation.rs
│   ├── report.rs
│   ├── join.rs
│   ├── watch.rs
│   └── lib.rs
│
├── data/                    # Données de test
│   ├── test.csv
│   ├── test.json
│   └── test_delimited.txt
│
├── tests/
│   └── integration_test.rs
│
└── Cargo.toml
```

---

## 🧪 Tests

```bash
# Compilation
cargo build --release

# Tests
cargo test

# Exemple d'utilisation
./target/release/datapipe --config test_pipeline.toml
```

---

## 👥 Équipe - Groupe 6 ENSP Yaoundé 2024-2025

| # | Nom | Rôle |
|---|-----|------|
| 01 | TSEFACK CALVIN KLEIN | Chef de Projet - Architecture |
| 02 | NZEUTEM DOMMOE EUNICE FELIXTINE | Lecteur CSV |
| 03 | DIOM LUCRAINE LETHICIA FIEN | Lecteurs JSON & Délimité |
| 04 | ASSONGUE TATANG MURIEL | Transformations Rename/Filter |
| 05 | NOLACK KAWUNJIBI FRANGE PARKER | Transformations Cast/Compute/Drop |
| 06 | NGLITANG RUBEN | Écrivains CSV & JSON |
| 07 | NGANSOP NGOUABOU FREDI LOIK | Écrivain JSONL & Tests |
| 08 | DONFACK KEUNANG VLADIMIR GEORGES | Stats & Rapports |
| 09 | ATEKOUMBO EXCEL DEXTE JORIS | JOIN & Dry-run |
| 10 | NJOH MASSANGO ADOLPHE MACDEAUVILLE | Watch & Docs |

---

## 📝 Exemple Complet

**Input CSV:**
```
nom_complet,departement,salaire,mot_de_passe
Alice Martin,Informatique,5000,secret123
Bob Dupont,RH,4000,pass456
Charlie Bernard,Informatique,5500,pwd789
```

**Pipeline.toml:**
```toml
[source]
format = "csv"
path = "data/test.csv"

[destination]
format = "json"
path = "output/resultat.json"

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
```

**Output JSON:**
```json
[
  {
    "nom": "Alice Martin",
    "departement": "Informatique",
    "salaire": 5000
  },
  {
    "nom": "Charlie Bernard",
    "departement": "Informatique",
    "salaire": 5500
  }
]
```

---

## 📊 Performance

- ⚡ Traitement en streaming (mémoire O(1) pour JSONL)
- 🚀 Jusqu'à 1M records/seconde sur CPU moderne
- 💾 Fichiers de 10 GB = <100 MB RAM

---

**Bon traitement de données! 🎉**
