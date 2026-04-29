# DataPipe — Outil ETL en Rust

**DataPipe** est un outil ETL (Extract, Transform, Load) en ligne de commande développé en Rust.
Il lit un fichier structuré (CSV, JSON ou texte délimité), applique des transformations déclarées dans un fichier TOML, puis écrit le résultat dans le format souhaité.

> Projet 6 — Programmation Système | Groupe 6 — ENSPD-GLO4 | 2025-2026

---

## Table des matières

1. [Prérequis et installation](#1-prérequis-et-installation)
2. [Compilation](#2-compilation)
3. [Utilisation rapide](#3-utilisation-rapide)
4. [Structure du fichier de configuration](#4-structure-du-fichier-de-configuration)
5. [Formats de source supportés](#5-formats-de-source-supportés)
6. [Transformations disponibles](#6-transformations-disponibles)
7. [Formats de destination supportés](#7-formats-de-destination-supportés)
8. [Modes d'exécution](#8-modes-dexécution)
9. [Validation de schéma](#9-validation-de-schéma)
10. [Rapport de statistiques](#10-rapport-de-statistiques)
11. [Exemples inclus](#11-exemples-inclus)
12. [Tests](#12-tests)
13. [Structure du projet](#13-structure-du-projet)
14. [Dépendances](#14-dépendances)
15. [Équipe](#15-équipe)

---

## 1. Prérequis et installation

- **Rust** 1.75.0 ou supérieur — https://rustup.rs
- Aucune autre dépendance système requise

```bash
# Vérifier la version Rust
rustc --version
cargo --version
```

---

## 2. Compilation

```bash
# Mode debug
cargo build

# Mode release (optimisé — recommandé)
cargo build --release
```

L'exécutable se trouve dans `target/release/datapipe` (Windows : `target\release\datapipe.exe`).

---

## 3. Utilisation rapide

**Linux / macOS :**
```bash
# Exécution normale
./target/release/datapipe --config pipeline.toml

# Aperçu sans écriture (dry-run)
./target/release/datapipe --config pipeline.toml --dry-run

# Surveillance automatique des changements
./target/release/datapipe --config pipeline.toml --watch --interval 60

# Afficher l'aide
./target/release/datapipe --help
```

**Windows (PowerShell ou cmd) :**
```cmd
# Exécution normale
target\release\datapipe.exe --config pipeline.toml

# Aperçu sans écriture (dry-run)
target\release\datapipe.exe --config pipeline.toml --dry-run

# Surveillance automatique des changements
target\release\datapipe.exe --config pipeline.toml --watch --interval 60

# Afficher l'aide
target\release\datapipe.exe --help
```

---

## 4. Structure du fichier de configuration

Toute la logique du pipeline est décrite dans un fichier `.toml` :

```toml
# Source de données
[source]
format    = "csv"               # "csv", "json" ou "delimited"
path      = "data/employes.csv"
delimiter = ","                 # requis pour csv et delimited

# Destination
[destination]
format = "json"                 # "csv", "json" ou "jsonl"
path   = "output/resultat.json"

# Transformations (optionnelles, appliquées dans l'ordre)
[[transforms]]
type = "rename"
from = "nom_complet"
to   = "nom"

[[transforms]]
type     = "filter"
column   = "departement"
operator = "="
value    = "Informatique"

[[transforms]]
type        = "cast"
column      = "salaire"
target_type = "number"

[[transforms]]
type       = "compute"
new_column = "prime"
expression = "salaire * 0.1"

[[transforms]]
type   = "drop"
column = "mot_de_passe"

# Validation de schéma (optionnelle)
[schema]
required_columns = ["nom", "salaire"]

[schema.column_types]
salaire = "float"
```

---

## 5. Formats de source supportés

### CSV

```toml
[source]
format    = "csv"
path      = "data/fichier.csv"
delimiter = ","      # ou ";" pour les exports Excel français
```

### JSON

Le fichier doit contenir un tableau d'objets `[{...}, {...}]`.

```toml
[source]
format = "json"
path   = "data/fichier.json"
```

### Texte délimité (Delimited)

```toml
[source]
format    = "delimited"
path      = "data/fichier.txt"
delimiter = "\t"     # tabulation ; ou "|", etc.
```

---

## 6. Transformations disponibles

### `rename` — Renommer une colonne

```toml
[[transforms]]
type = "rename"
from = "ancien_nom"
to   = "nouveau_nom"
```

### `filter` — Filtrer des enregistrements

Garde uniquement les enregistrements satisfaisant la condition.

```toml
[[transforms]]
type     = "filter"
column   = "age"
operator = ">="      # = | != | < | <= | > | >= | contains
value    = "18"
```

### `cast` — Convertir le type d'une colonne

```toml
[[transforms]]
type        = "cast"
column      = "salaire"
target_type = "number"    # "number" | "boolean" | "string"
```

> **Important :** effectuer un `cast` avant d'appliquer des opérateurs numériques (`>`, `<`…) sur des colonnes lues en CSV (toutes les valeurs CSV sont du texte par défaut).

### `compute` — Calculer une nouvelle colonne

```toml
[[transforms]]
type       = "compute"
new_column = "prime"
expression = "salaire * 0.1"   # opérateurs : * + - / concat
```

### `drop` — Supprimer une colonne

```toml
[[transforms]]
type   = "drop"
column = "mot_de_passe"
```

---

## 7. Formats de destination supportés

| Format  | Usage                              | Comportement                                  |
|---------|------------------------------------|-----------------------------------------------|
| `csv`   | Compatibilité Excel/LibreOffice    | En-têtes auto-générées à la première ligne    |
| `json`  | APIs web, petits fichiers          | Tableau JSON complet chargé en mémoire        |
| `jsonl` | Gros volumes, streaming            | Une ligne JSON par enregistrement (BufWriter) |

---

## 8. Modes d'exécution

| Option         | Description                                                             |
|----------------|-------------------------------------------------------------------------|
| *(aucune)*     | Traitement complet : lecture → transformations → écriture               |
| `--dry-run`    | Simulation : affiche config + aperçu des 5 premiers records, sans écrire|
| `--watch`      | Surveille le fichier `.toml` et relance le pipeline à chaque modification|
| `--interval N` | Intervalle de surveillance en secondes (défaut : 30)                    |

---

## 9. Validation de schéma

Lorsqu'une section `[schema]` est présente dans la configuration, DataPipe vérifie pour chaque enregistrement :

- **Colonnes requises** : la colonne doit être présente et non vide.
- **Types** : `integer`, `float`, `boolean`, `string`.

Les erreurs de validation sont signalées dans la console (`stderr`) sans interrompre le traitement.

---

## 10. Rapport de statistiques

À la fin de chaque exécution, un rapport s'affiche :

```
[OK] Pipeline terminé!

════════════════════════════════════════════════════
         RAPPORT D'EXÉCUTION — DATAPIPE
════════════════════════════════════════════════════
     PIPELINE
────────────────────────────────────────────────────
    Source        : data/employees.csv (csv)
    Destination   : output/resultat.json (json)
    Transformations configurées : 4
────────────────────────────────────────────────────
     HORODATAGE
────────────────────────────────────────────────────
    Début         : 2026-04-29 10:00:00 UTC
    Fin           : 2026-04-29 10:00:00 UTC
    Durée         : 12ms
────────────────────────────────────────────────────
     RECORDS
────────────────────────────────────────────────────
    Lus              : 25
    Transformés      : 9
    Filtrés           : 16
    Écrits            : 9
    Erreurs           : 0
────────────────────────────────────────────────────
     PERFORMANCE
────────────────────────────────────────────────────
    Débit             : 2083 records/s
    Volume estimé     : 6.2 Ko
════════════════════════════════════════════════════
    STATUT : SUCCÈS — pipeline terminé sans erreur ni anomalie
════════════════════════════════════════════════════
```

---

## 11. Exemples inclus

Les fichiers du dossier `examples/` sont prêts à l'emploi :

```bash
# Employés : CSV → JSON avec filtrage et calcul de prime
./target/release/datapipe --config examples/employes_transform.toml

# Transactions : CSV → JSONL (streaming)
./target/release/datapipe --config examples/transactions_stream.toml

# Produits en promotion : JSON → CSV
./target/release/datapipe --config examples/produits_promo.toml

# Grandes villes : TSV → JSON avec validation
./target/release/datapipe --config examples/villes_grandes.toml
```

Données de démonstration disponibles dans `data/` :
- `employees.csv`, `transactions.csv`, `contacts.csv` (CSV)
- `produits.json`, `commandes.json` (JSON)
- `villes.txt` (TSV)

---

## 12. Tests

```bash
# Lancer tous les tests
cargo test

# Tests d'intégration uniquement
cargo test --test integration_test
cargo test --test integration_tests_extended

# Tests unitaires uniquement
cargo test --lib
```

Résultats attendus : **124 tests**, 0 échec.

```
test result: ok. 40 passed; 0 failed   (tests unitaires lib)
test result: ok. 42 passed; 0 failed   (tests unitaires bin)
test result: ok. 11 passed; 0 failed   (csv_test)
test result: ok. 14 passed; 0 failed   (integration_test)
test result: ok. 14 passed; 0 failed   (integration_tests_extended)
test result: ok.  3 passed; 0 failed   (doc-tests)
```

---

## 13. Structure du projet

```
datapipe/
├── Cargo.toml                     # Manifest du projet et dépendances
├── pipeline.toml                  # Configuration d'exemple (racine)
├── src/
│   ├── main.rs                    # Point d'entrée CLI (clap)
│   ├── lib.rs                     # Exports publics du crate
│   ├── config.rs                  # Désérialisation de la configuration TOML
│   ├── pipeline.rs                # Orchestrateur principal ETL
│   ├── stats.rs                   # Collecte et affichage des statistiques
│   ├── validation.rs              # Validation de schéma
│   ├── report.rs                  # Génération de rapport HTML (bonus)
│   ├── join.rs                    # Jointures entre sources (bonus)
│   ├── watch.rs                   # Mode surveillance automatique
│   ├── reader/
│   │   ├── mod.rs                 # Trait SourceReader, type Record
│   │   ├── csv_reader.rs          # Lecteur CSV
│   │   ├── json_reader.rs         # Lecteur JSON
│   │   └── delimited_reader.rs    # Lecteur texte délimité
│   ├── transform/
│   │   ├── mod.rs                 # Trait Transform, apply_chain()
│   │   ├── rename.rs              # Transformation rename
│   │   ├── filter.rs              # Transformation filter
│   │   ├── cast.rs                # Transformation cast
│   │   ├── compute.rs             # Transformation compute
│   │   ├── drop.rs                # Transformation drop
│   │   └── factory.rs             # Fabrique de transformations
│   └── writer/
│       ├── mod.rs                 # Trait SinkWriter
│       ├── csv_writer.rs          # Écrivain CSV
│       ├── json_writer.rs         # Écrivain JSON
│       ├── jsonl_writer.rs        # Écrivain JSONL (streaming BufWriter)
│       └── factory.rs             # Fabrique d'écrivains
├── tests/
│   ├── csv_test.rs                # Tests unitaires du lecteur CSV
│   ├── integration_test.rs        # Tests d'intégration principaux
│   └── integration_tests_extended.rs  # Tests d'intégration étendus
├── data/                          # Fichiers de données de démonstration
└── examples/                      # Configurations TOML d'exemple
```

---

## 14. Dépendances

| Crate        | Version | Rôle                                                  |
|--------------|---------|-------------------------------------------------------|
| `csv`        | 1.3     | Lecture et écriture de fichiers CSV                   |
| `serde`      | 1.0     | Framework de sérialisation/désérialisation            |
| `serde_json` | 1.0     | Traitement JSON, type `Value` universel               |
| `toml`       | 0.8     | Parsing des fichiers de configuration `.toml`         |
| `clap`       | 4.0     | Parsing des arguments de ligne de commande            |
| `anyhow`     | 1.0     | Gestion ergonomique des erreurs                       |
| `indexmap`   | 2.0     | HashMap ordonnée (préserve l'ordre des colonnes)      |
| `tempfile`   | 3.0     | Fichiers temporaires pour les tests (dev uniquement)  |

---

## 15. Équipe

| Rôle                              | Membre                        |
|-----------------------------------|-------------------------------|
| Chef de projet / pipeline.rs      | TSEFACK CALVIN KLEIN          |
| Transformations (trait + impléms) | ASSONGUE                      |
| Statistiques d'exécution          | DONFACK (#08)                 |
| Écrivains JSON/JSONL/factory      | NGANSOP NGOUABOU FREDI LOIK   |
| Lecteurs CSV/JSON/Délimité        | Équipe — Groupe 6             |
| Tests & intégration               | Équipe — Groupe 6             |

---

*DataPipe v0.1.0 — Groupe 6, ENSPD-GLO4, 2025-2026*
