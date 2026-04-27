# 📘 README — Travail de NGANSOP NGOUABOU FREDI LOIK (#07)

**Projet** : DataPipe — Outil ETL en Rust  
**Équipe** : Groupe 6, ENSPD Douala — Génie Logiciel  
**Rôle** : Écrivain JSONL, Fabrique d'écrivains, Tests d'intégration

---

## Table des matières

1. [Vue d'ensemble de ma contribution](#1-vue-densemble-de-ma-contribution)
2. [Concepts théoriques fondamentaux](#2-concepts-théoriques-fondamentaux)
   - 2.1 [Le format JSONL (JSON Lines)](#21-le-format-jsonl-json-lines)
   - 2.2 [Le Factory Pattern](#22-le-factory-pattern)
   - 2.3 [Le trait SinkWriter et le polymorphisme](#23-le-trait-sinkwriter-et-le-polymorphisme)
   - 2.4 [Le BufWriter et les performances I/O](#24-le-bufwriter-et-les-performances-io)
   - 2.5 [Tests unitaires vs Tests d'intégration](#25-tests-unitaires-vs-tests-dintégration)
3. [Fichiers créés et leur rôle](#3-fichiers-créés-et-leur-rôle)
4. [Explication détaillée du code](#4-explication-détaillée-du-code)
   - 4.1 [src/writer/mod.rs — Le trait SinkWriter](#41-srcwritermodrs--le-trait-sinkwriter)
   - 4.2 [src/writer/jsonl_writer.rs — L'écrivain JSONL](#42-srcwriterjsonl_writerrs--lécrivain-jsonl)
   - 4.3 [src/writer/factory.rs — La fabrique](#43-srcwriterfactoryrs--la-fabrique)
   - 4.4 [tests/integration_test.rs — Les tests](#44-testsintegration_testrs--les-tests)
5. [Erreurs classiques à éviter](#5-erreurs-classiques-à-éviter)
6. [Comment exécuter les tests](#6-comment-exécuter-les-tests)
7. [Intégration avec le reste du projet](#7-intégration-avec-le-reste-du-projet)

---

## 1. Vue d'ensemble de ma contribution

Dans l'architecture ETL (**E**xtract → **T**ransform → **L**oad), je suis responsable du **L** — le chargement final des données vers leur destination.

```
[Source CSV/JSON/TXT]    [Transformations]    [Destination]
   NZEUTEM, DIOM    →   ASSONGUE, NOLACK   →   NGLITANG + NGANSOP
                                                     ↑
                                           C'est mon périmètre
```

### Fichiers sous ma responsabilité

| Fichier | Rôle | Statut |
|---------|------|--------|
| `src/writer/mod.rs` | Définit le trait `SinkWriter` (contrat) | ✅ Implémenté |
| `src/writer/jsonl_writer.rs` | Écrivain JSONL streaming | ✅ Implémenté + 9 tests unitaires |
| `src/writer/factory.rs` | Fabrique d'écrivains (Factory Pattern) | ✅ Implémenté + 5 tests unitaires |
| `tests/integration_test.rs` | Tests d'intégration bout en bout | ✅ 14 tests |
| `src/lib.rs` | Point d'entrée bibliothèque (nouveau) | ✅ Créé |
| `data/employees.csv` | Données de test réalistes | ✅ Créé |
| `data/test.json` | Données JSON de test | ✅ Créé |
| `data/test_delimited.txt` | Données délimitées de test | ✅ Créé |

**Bilan** : 28 tests passent (14 unitaires + 14 intégration) ✅

---

## 2. Concepts théoriques fondamentaux

### 2.1 Le format JSONL (JSON Lines)

#### Définition

JSONL (JSON Lines), aussi appelé NDJSON (Newline-Delimited JSON), est un format de fichier texte où **chaque ligne est un objet JSON autonome et valide**.

#### Fichier JSON classique vs Fichier JSONL

**JSON classique (tableau) :**
```json
[
  {"nom": "Jean", "age": 25, "ville": "Douala"},
  {"nom": "Marie", "age": 30, "ville": "Yaoundé"},
  {"nom": "Paul", "age": 35, "ville": "Bafoussam"}
]
```

**JSONL (JSON Lines) :**
```jsonl
{"nom":"Jean","age":25,"ville":"Douala"}
{"nom":"Marie","age":30,"ville":"Yaoundé"}
{"nom":"Paul","age":35,"ville":"Bafoussam"}
```

#### Pourquoi JSONL est supérieur pour l'ETL ?

| Critère | JSON classique | JSONL |
|---------|---------------|-------|
| Mémoire nécessaire | O(n) — tout le fichier | O(1) — une ligne à la fois |
| Début de traitement | Après lecture totale | Immédiat (ligne 1) |
| Fichier corrompu | Tout le fichier invalide | Seulement la ligne corrompue |
| Ajout de données | Re-écriture complète | `echo '{"x":1}' >> fichier.jsonl` |
| Fichiers de 10 Go+ | ❌ Impossible en RAM | ✅ Fonctionne |

#### Règle absolue du format JSONL

> **1 ligne = 1 objet JSON = aucun retour à la ligne INTERNE à l'objet**

C'est pourquoi on utilise `serde_json::to_string()` (compact) et **jamais** `to_writer_pretty()` (indenté). Le pretty-print ajouterait des `\n` à l'intérieur de l'objet, cassant la structure JSONL.

**Erreur classique :**
```
// Pretty-print invalide pour JSONL :
{
  "nom": "Jean",   ← Ce \n interne invalide le JSONL
  "age": 25
}
```

**Correct :**
```
{"nom":"Jean","age":25}
```

---

### 2.2 Le Factory Pattern

#### Le problème sans Factory

Imaginez que l'orchestrateur (`pipeline.rs`) doive choisir lui-même le bon écrivain :

```rust
// ❌ MAUVAISE APPROCHE — couplage fort
fn creer_writer(config: &DestinationConfig) -> Box<dyn SinkWriter> {
    if config.format == "csv" {
        Box::new(CsvSinkWriter::new(...))  // pipeline.rs doit connaître CsvSinkWriter
    } else if config.format == "json" {
        Box::new(JsonSinkWriter::new(...)) // pipeline.rs doit connaître JsonSinkWriter
    } else if config.format == "jsonl" {
        Box::new(JsonLinesSinkWriter::new(...)) // etc.
    } else {
        panic!("Format inconnu")
    }
}
```

**Problèmes :**
1. `pipeline.rs` est couplé à tous les types concrets d'écrivains
2. Pour ajouter "parquet", il faut modifier `pipeline.rs` → violation du principe Open/Closed
3. Difficile à tester (chaque cas dépend d'autres modules)

#### La solution : Factory Pattern

```rust
// ✅ BONNE APPROCHE — factory encapsule la création
// Dans factory.rs (mon fichier) :
pub fn create_writer(config: &DestinationConfig) -> Result<Box<dyn SinkWriter>> {
    match config.format.as_str() {
        "csv"   => Ok(Box::new(CsvSinkWriter::new(&config.path)?)),
        "json"  => Ok(Box::new(JsonSinkWriter::new(&config.path))),
        "jsonl" => Ok(Box::new(JsonLinesSinkWriter::new(&config.path)?)),
        fmt     => Err(anyhow!("Format '{}' non reconnu", fmt)),
    }
}

// Dans pipeline.rs (TSEFACK) :
let mut writer = create_writer(&config.destination)?; // Simple !
```

**Avantages :**
1. `pipeline.rs` ne connaît que `create_writer()` et `Box<dyn SinkWriter>` — découplage total
2. Pour ajouter "parquet" : une seule ligne dans `factory.rs`, rien d'autre à changer
3. Chaque partie est testable indépendamment

#### Représentation visuelle du Factory Pattern

```
                    [pipeline.rs]
                         |
                  create_writer(config)
                         |
                    [factory.rs]   ← MON FICHIER
                   /     |      \
                  /      |       \
        CsvWriter  JsonWriter  JsonlWriter   ← tous implémentent SinkWriter
             \         |         /
              \        |        /
               [Box<dyn SinkWriter>]
                         |
                    pipeline.rs utilise
                    write_record() et finalize()
                    SANS savoir quel type concret
```

---

### 2.3 Le trait SinkWriter et le polymorphisme

#### Qu'est-ce qu'un trait en Rust ?

Un trait en Rust est l'équivalent d'une **interface** en Java ou C#. Il définit un **contrat** : "tout type qui implémente ce trait doit fournir ces méthodes".

```rust
// Mon contrat (mon fichier src/writer/mod.rs) :
pub trait SinkWriter {
    fn write_record(&mut self, record: &Record) -> anyhow::Result<()>;
    fn finalize(&mut self) -> anyhow::Result<()>;
}
```

Toute struct qui implémente `SinkWriter` doit fournir ces deux méthodes.

#### Polymorphisme avec `Box<dyn SinkWriter>`

`Box<dyn SinkWriter>` est un **trait object** — un pointeur vers n'importe quelle struct qui implémente `SinkWriter`, résolu au moment de l'exécution (polymorphisme dynamique).

```rust
// Ces trois appels sont IDENTIQUES du point de vue du pipeline :
let mut w1: Box<dyn SinkWriter> = Box::new(CsvSinkWriter::new("out.csv")?);
let mut w2: Box<dyn SinkWriter> = Box::new(JsonSinkWriter::new("out.json"));
let mut w3: Box<dyn SinkWriter> = Box::new(JsonLinesSinkWriter::new("out.jsonl")?);

// Dans tous les cas, l'appel est identique :
w1.write_record(&record)?;
w2.write_record(&record)?;
w3.write_record(&record)?;
```

Le pipeline ne sait pas quelle implémentation il utilise. C'est la définition du polymorphisme.

#### Comparaison Java / Rust

```java
// Java
interface SinkWriter {
    void writeRecord(Map<String, Object> record) throws IOException;
    void finalize() throws IOException;
}

class JsonLinesSinkWriter implements SinkWriter { ... }
```

```rust
// Rust (équivalent)
pub trait SinkWriter {
    fn write_record(&mut self, record: &Record) -> Result<()>;
    fn finalize(&mut self) -> Result<()>;
}

struct JsonLinesSinkWriter { ... }
impl SinkWriter for JsonLinesSinkWriter { ... }
```

---

### 2.4 Le BufWriter et les performances I/O

#### Pourquoi les I/O disque sont coûteuses ?

Chaque écriture directe sur le disque (`write()` syscall) coûte du temps car elle traverse :
1. Le code Rust
2. Le noyau Linux (syscall)
3. Le driver disque
4. Le contrôleur physique

Pour un fichier de 100 000 lignes, 100 000 syscalls = très lent.

#### BufWriter : regrouper les écritures

```rust
// Sans BufWriter : 1 syscall par ligne = lent ❌
let file = File::create("output.jsonl")?;
writeln!(file, "{}", json_line)?; // syscall immédiat

// Avec BufWriter : 1 syscall tous les 8192 octets = rapide ✅
let file = File::create("output.jsonl")?;
let mut writer = BufWriter::new(file); // buffer de 8192 octets
writeln!(writer, "{}", json_line)?; // écrit dans le buffer RAM
// Le syscall réel n'arrive que quand le buffer est plein (8192 octets)
// ou lors du flush() explicite
```

#### Pourquoi `finalize()` est indispensable

```
Situation SANS finalize() — BUG CLASSIQUE :

Étape 1 : write_record() × 100 → buffer RAM : 5000 octets (< 8192)
Étape 2 : Programme termine
Étape 3 : BufWriter est détruit → buffer RAM libéré SANS être écrit
Résultat : Fichier vide sur le disque ❌

Situation AVEC finalize() — CORRECT :

Étape 1 : write_record() × 100 → buffer RAM : 5000 octets
Étape 2 : finalize() → flush() → syscall → 5000 octets sur le disque
Étape 3 : Programme termine
Résultat : Fichier complet sur le disque ✅
```

**Analogie** : BufWriter est comme un employé postal. Sans `flush()`, il garde les lettres dans son sac mais ne les poste jamais. `flush()` lui dit "va poster maintenant, même si le sac n'est pas plein".

---

### 2.5 Tests unitaires vs Tests d'intégration

#### Tests unitaires (dans `src/`)

Vérifient un **composant isolé**.

```rust
// Dans src/writer/jsonl_writer.rs
#[test]
fn test_ecriture_un_record() {
    // Je crée directement un JsonLinesSinkWriter, sans passer par la factory
    // Je teste UNIQUEMENT le comportement de write_record() + finalize()
    let mut writer = JsonLinesSinkWriter::new(&path).unwrap();
    writer.write_record(&record).unwrap();
    writer.finalize().unwrap();
    // Vérification...
}
```

#### Tests d'intégration (dans `tests/`)

Vérifient **plusieurs composants ensemble**.

```rust
// Dans tests/integration_test.rs
#[test]
fn test_pipeline_csv_vers_jsonl() {
    // Je simule un pipeline complet :
    // Records "lus" → transformations → écriture via factory → vérification du fichier
    let mut writer = create_writer(&config)?; // ← utilise la factory
    for record in records_transformes {
        writer.write_record(&record)?;
    }
    writer.finalize()?;
    // Vérification du fichier de sortie final
}
```

#### Placement des fichiers de tests en Rust

```
src/
├── writer/
│   ├── jsonl_writer.rs  ← tests unitaires avec #[cfg(test)] mod tests { ... }
│   └── factory.rs       ← tests unitaires avec #[cfg(test)] mod tests { ... }
tests/
└── integration_test.rs  ← tests d'intégration (boîte noire, accès uniquement pub)
```

Règle : les tests unitaires ont accès aux membres privés (`pub(crate)`). Les tests d'intégration n'ont accès qu'aux membres `pub` — exactement comme un utilisateur externe.

---

## 3. Fichiers créés et leur rôle

```
DATAPIPE/
├── src/
│   ├── lib.rs                    ← [NOUVEAU - NGANSOP] Point d'entrée bibliothèque
│   └── writer/
│       ├── mod.rs                ← [NGANSOP] Trait SinkWriter + ré-exports
│       ├── jsonl_writer.rs       ← [NGANSOP] Écrivain JSONL streaming
│       ├── factory.rs            ← [NGANSOP] Fabrique d'écrivains
│       ├── csv_writer.rs         ← [stub NGANSOP, compléter par NGLITANG]
│       └── json_writer.rs        ← [stub NGANSOP, compléter par NGLITANG]
├── tests/
│   └── integration_test.rs       ← [NGANSOP] 14 tests d'intégration
└── data/
    ├── employees.csv             ← [NGANSOP] Données de test réalistes
    ├── test.json                 ← [NGANSOP] Données JSON de test
    └── test_delimited.txt        ← [NGANSOP] Données délimitées de test
```

> **Note sur csv_writer.rs et json_writer.rs** : Ces fichiers appartiennent à NGLITANG (#06). Comme ils n'existaient pas encore et que `factory.rs` en a besoin pour compiler, j'ai créé des implémentations fonctionnelles complètes. NGLITANG peut les affiner ou les remplacer par ses propres implémentations.

---

## 4. Explication détaillée du code

### 4.1 `src/writer/mod.rs` — Le trait SinkWriter

Ce fichier déclare le **contrat** que tous les écrivains doivent respecter.

```rust
pub trait SinkWriter {
    fn write_record(&mut self, record: &Record) -> anyhow::Result<()>;
    fn finalize(&mut self) -> anyhow::Result<()>;
}
```

**Pourquoi `&mut self` ?**  
`write_record` modifie l'état interne de l'écrivain (position dans le fichier, buffer, compteur). Rust requiert `&mut self` pour toute méthode qui mute l'état.

**Pourquoi `anyhow::Result<()>` ?**  
Les opérations I/O peuvent échouer (disque plein, permissions). `anyhow::Result` permet de propager l'erreur avec `?` jusqu'à l'appelant qui décidera quoi faire.

---

### 4.2 `src/writer/jsonl_writer.rs` — L'écrivain JSONL

#### Structure

```rust
pub struct JsonLinesSinkWriter {
    writer: BufWriter<File>,    // Buffer + fichier de destination
    records_written: usize,     // Compteur pour le debug
}
```

#### Constructeur `new()`

```rust
pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
    // 1. Créer les répertoires parents si nécessaires
    if let Some(parent) = path.as_ref().parent() {
        std::fs::create_dir_all(parent)?;
    }
    // 2. Créer le fichier
    let file = File::create(&path)?;
    // 3. Envelopper dans BufWriter
    Ok(JsonLinesSinkWriter {
        writer: BufWriter::new(file),
        records_written: 0,
    })
}
```

**`<P: AsRef<Path>>`** : Generic bound. `P` peut être `&str`, `String`, `PathBuf`, `Path`... tout type convertible en référence vers un chemin. Plus flexible qu'accepter uniquement `&str`.

#### `write_record()`

```rust
fn write_record(&mut self, record: &Record) -> Result<()> {
    // Sérialisation JSON COMPACTE (obligatoire pour JSONL)
    let json_line = serde_json::to_string(record)?;
    // Écriture d'une ligne avec saut de ligne
    writeln!(self.writer, "{}", json_line)?;
    self.records_written += 1;
    Ok(())
}
```

**Flux de données :**
```
Record (HashMap<String, Value>)
    ↓  serde_json::to_string()
'{"age":25,"nom":"Jean"}'  (String)
    ↓  writeln!(self.writer, ...)
buffer BufWriter (8Ko en RAM)
    ↓  (quand buffer plein ou flush)
fichier .jsonl sur disque
```

#### `finalize()`

```rust
fn finalize(&mut self) -> Result<()> {
    self.writer.flush()?;  // Force l'écriture du buffer résiduel
    Ok(())
}
```

---

### 4.3 `src/writer/factory.rs` — La fabrique

```rust
pub fn create_writer(config: &DestinationConfig) -> Result<Box<dyn SinkWriter>> {
    match config.format.as_str() {
        "csv"   => Ok(Box::new(CsvSinkWriter::new(&config.path, None)?)),
        "json"  => Ok(Box::new(JsonSinkWriter::new(&config.path))),
        "jsonl" => Ok(Box::new(JsonLinesSinkWriter::new(&config.path)?)),
        fmt     => Err(anyhow!(
            "Format '{}' non reconnu. Formats supportés : csv, json, jsonl", fmt
        )),
    }
}
```

**`Box::new(...)`** : Alloue la struct sur le tas (heap). Nécessaire car la taille de `Box<dyn SinkWriter>` est connue à la compilation (c'est un fat pointer : data + vtable), même si la struct concrète a une taille variable.

**`Box<dyn SinkWriter>`** : Trait object. Le `dyn` indique la résolution dynamique (au runtime). C'est le polymorphisme en Rust.

---

### 4.4 `tests/integration_test.rs` — Les tests

#### Organisation par sections

```rust
// SECTION 1 : Tests JSONL Writer (le composant de NGANSOP)
//   - test_jsonl_writer_ecriture_basique
//   - test_jsonl_format_strict_une_ligne_par_record

// SECTION 2 : Tests Factory (l'autre composant de NGANSOP)
//   - test_factory_format_jsonl
//   - test_factory_format_json
//   - test_factory_format_csv
//   - test_factory_format_inconnu_erreur

// SECTION 3 : Tests d'intégration pipeline bout en bout
//   - test_pipeline_records_en_memoire_vers_jsonl
//   - test_pipeline_1000_records_jsonl         (test de charge)
//   - test_pipeline_avec_filtre_records_rejetes
//   - test_pipeline_rename_compute_drop_vers_jsonl
//   - test_statistiques_pipeline_simule
//   - test_coherence_json_vs_jsonl
//   - test_robustesse_valeurs_speciales
//   - test_polymorphisme_sink_writer
```

#### Résultat : 14/14 tests passent ✅

---

## 5. Erreurs classiques à éviter

### ❌ Erreur 1 : Oublier `finalize()`

```rust
// FAUX : le buffer n'est jamais écrit sur le disque
let mut writer = JsonLinesSinkWriter::new("output.jsonl")?;
writer.write_record(&record)?;
// finalize() oublié → fichier vide ou tronqué !
```

```rust
// CORRECT :
let mut writer = JsonLinesSinkWriter::new("output.jsonl")?;
writer.write_record(&record)?;
writer.finalize()?; // ← OBLIGATOIRE
```

### ❌ Erreur 2 : Utiliser `to_writer_pretty` pour JSONL

```rust
// FAUX : génère du JSON multilignes → invalide pour JSONL
let json = serde_json::to_string_pretty(record)?;
writeln!(writer, "{}", json)?;
// Résultat : {
//   "nom": "Jean",    ← ce \n casse JSONL !
//   "age": 25
// }
```

```rust
// CORRECT : JSON compact sur une ligne
let json = serde_json::to_string(record)?;
writeln!(writer, "{}", json)?;
// Résultat : {"nom":"Jean","age":25}
```

### ❌ Erreur 3 : Utiliser `unwrap()` en production

```rust
// FAUX : un fichier manquant fait crasher tout le programme
let file = File::create("output.jsonl").unwrap(); // panique si erreur !
```

```rust
// CORRECT : propager l'erreur avec ?
let file = File::create("output.jsonl")
    .with_context(|| "Impossible de créer output.jsonl")?;
```

### ❌ Erreur 4 : Ne pas créer le répertoire parent

```rust
// FAUX : si "output/" n'existe pas, File::create échoue
let file = File::create("output/results.jsonl")?; // Err si "output/" absent
```

```rust
// CORRECT : créer le répertoire parent d'abord
std::fs::create_dir_all("output/")?;
let file = File::create("output/results.jsonl")?;
```

### ❌ Erreur 5 : Ne pas ajouter `lib.rs` pour les tests d'intégration

Sans `src/lib.rs`, les tests dans `tests/` ne peuvent pas accéder aux modules.

```
// ERREUR dans tests/integration_test.rs :
use datapipe::writer::factory::create_writer; // error: unresolved import
```

```rust
// CORRECTION : ajouter src/lib.rs avec :
pub mod writer;
pub mod config;
// etc.
```

---

## 6. Comment exécuter les tests

### Prérequis
```bash
# Rust et Cargo doivent être installés
rustc --version  # >= 1.70.0
cargo --version
```

### Lancer tous les tests de NGANSOP

```bash
# Tests unitaires uniquement (dans src/)
cargo test --lib -- writer

# Tests d'intégration uniquement (dans tests/)
cargo test --test integration_test

# Tous les tests (unitaires + intégration)
cargo test

# Avec sortie détaillée
cargo test -- --nocapture

# Un test spécifique
cargo test test_mille_records
cargo test test_pipeline_avec_filtre
```

### Résultat attendu

```
running 14 tests
test writer::factory::tests::test_factory_cree_jsonl ... ok
test writer::factory::tests::test_factory_cree_json ... ok
test writer::factory::tests::test_format_inconnu_retourne_erreur ... ok
test writer::factory::tests::test_format_vide_retourne_erreur ... ok
test writer::factory::tests::test_jsonl_format_valide ... ok
test writer::jsonl_writer::tests::test_compteur_records ... ok
test writer::jsonl_writer::tests::test_ecriture_plusieurs_records ... ok
test writer::jsonl_writer::tests::test_creation_repertoire_parent ... ok
test writer::jsonl_writer::tests::test_ecriture_un_record ... ok
test writer::jsonl_writer::tests::test_finalize_sans_record ... ok
test writer::jsonl_writer::tests::test_format_compact_une_ligne_par_record ... ok
test writer::jsonl_writer::tests::test_record_vide ... ok
test writer::jsonl_writer::tests::test_types_mixtes ... ok
test writer::jsonl_writer::tests::test_mille_records ... ok

test result: ok. 14 passed; 0 failed; 0 ignored

running 14 tests
test test_factory_format_csv ... ok
test test_factory_format_inconnu_erreur ... ok
test test_coherence_json_vs_jsonl ... ok
...
test result: ok. 14 passed; 0 failed; 0 ignored
```

---

## 7. Intégration avec le reste du projet

### Comment TSEFACK (#01) utilise mon code dans `pipeline.rs`

```rust
// Dans src/pipeline.rs (TSEFACK)
use crate::writer::factory::create_writer;

pub fn run(config_path: &Path) -> Result<()> {
    let config = PipelineConfig::from_file(config_path)?;
    let mut stats = ExecutionStats::new();

    // Créer le lecteur (NZEUTEM/DIOM)
    let reader = create_reader(&config.source)?;

    // Créer les transformations (ASSONGUE/NOLACK)
    let transforms: Vec<Box<dyn Transform>> = config.transforms
        .iter()
        .map(create_transform)
        .collect::<Result<_>>()?;

    // Créer l'écrivain ← MON CODE (NGANSOP)
    let mut writer = create_writer(&config.destination)?;

    // Pipeline principal
    for result in reader.records() {
        let record = result?;
        stats.records_read += 1;

        // Appliquer les transformations
        let final_record = transforms.iter().fold(Some(record), |rec, t| {
            rec.and_then(|r| t.apply(r))
        });

        match final_record {
            Some(r) => {
                writer.write_record(&r)?;  // ← Mon SinkWriter
                stats.records_written += 1;
            }
            None => stats.records_filtered += 1,
        }
    }

    writer.finalize()?;  // ← Mon SinkWriter
    stats.stop();
    stats.print_report();
    Ok(())
}
```

### Ajouter un nouveau format de sortie (exemple : Parquet)

Pour ajouter le format Parquet à DataPipe, une seule modification dans mon fichier `factory.rs` :

```rust
// Ajouter la dépendance dans Cargo.toml :
// parquet = "51.0"

// Dans factory.rs, ajouter une branche :
"parquet" => {
    let writer = ParquetSinkWriter::new(&config.path)?;
    Ok(Box::new(writer))
}
```

**Aucun autre fichier à modifier** — c'est l'avantage du Factory Pattern et du trait `SinkWriter`.

---

## Récapitulatif pédagogique

| Concept | Fichier | Ce qu'il illustre |
|---------|---------|-------------------|
| Trait (interface) | `writer/mod.rs` | Abstraction, contrat d'API |
| Implémentation de trait | `jsonl_writer.rs` | Polymorphisme, RAII |
| Factory Pattern | `factory.rs` | Open/Closed principle, découplage |
| BufWriter | `jsonl_writer.rs` | Optimisation I/O, gestion de ressources |
| Tests unitaires | `jsonl_writer.rs`, `factory.rs` | Isolation, TDD |
| Tests d'intégration | `tests/integration_test.rs` | Collaboration entre modules |
| Gestion d'erreurs | Partout | `anyhow`, propagation avec `?` |
| Généricité | `new<P: AsRef<Path>>` | Generic bounds, flexibilité |

---

*NGANSOP NGOUABOU FREDI LOIK — Groupe 6 — ENSPD Douala*  
*Génie Logiciel — Programmation Système Rust — 2024-2025*
