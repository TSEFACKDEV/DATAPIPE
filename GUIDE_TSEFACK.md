# 🎯 Guide TSEFACK - Rôle d'Orchestrateur Principal

Bonsoir TSEFACK ! Tu es le **chef d'orchestre** de DataPipe. Ce guide explique ton rôle et comment tout s'articule.

---

## 🎼 Ton rôle

Tu dois faire fonctionner l'**ensemble du système**. Les autres équipes implémentent les briques, toi tu les assemles.

### Fichiers que tu gères :
1. **Cargo.toml** - Liste de dépendances 
2. **src/main.rs** - Interface utilisateur (CLI)
3. **src/config.rs** - Structure des fichiers TOML
4. **src/pipeline.rs** - Le cœur du système
5. **README.md** - Documentation pour les utilisateurs

---

## 🔌 Le diagramme du pipeline

```
UTILISATEUR
    ↓
    datapipe --config pipeline.toml
    ↓
┌─────────────────────────────────┐
│      main.rs (CLI parsing)      │  ← Parse les arguments
└──────────────┬──────────────────┘
               ↓
┌─────────────────────────────────┐
│     pipeline.rs::run()          │  ← TU ES ICI!
│                                 │
│  1. Charge la config TOML       │
│     └─→ config.rs::PipelineConfig::from_file()
│                                 │
│  2. Crée le lecteur             │
│     └─→ factory::create_reader()
│         ├─→ CsvReader (NZEUTEM)
│         ├─→ JsonReader (DIOM)
│         └─→ DelimitedReader (DIOM)
│                                 │
│  3. Crée les transformations    │
│     └─→ transform::factory::create_transform()
│         ├─→ RenameTransform (ASSONGUE)
│         ├─→ FilterTransform (ASSONGUE)
│         ├─→ CastTransform (NOLACK)
│         ├─→ ComputeTransform (NOLACK)
│         └─→ DropTransform (NOLACK)
│                                 │
│  4. Crée l'écrivain             │
│     └─→ writer::factory::create_writer()
│         ├─→ CsvWriter (NGLITANG)
│         ├─→ JsonWriter (NGLITANG)
│         └─→ JsonlWriter (NGANSOP)
│                                 │
│  5. Boucle principale (vous + 100 transformations)
│     pour chaque record:        │
│     ├─→ reader.records()        │
│     ├─→ transform.apply() x N   │
│     └─→ writer.write_record()   │
│                                 │
│  6. Finalise l'écriture         │
│     └─→ writer.finalize()       │
│                                 │
│  7. Affiche le rapport          │
│     └─→ stats.print_report()    │
└──────────────┬──────────────────┘
               ↓
           Fichier de sortie
```

---

## 📝 Code que tu as écrit

### Pipeline Run Function (L'orchestrateur)

```rust
pub fn run(config_path: &Path) -> Result<()> {
    // ÉTAPE 1: Charger config
    let config = PipelineConfig::from_file(config_path)?;
    let mut stats = ExecutionStats::new();
    
    // ÉTAPE 2: Créer lecteur
    let reader = create_reader(&config.source)?;
    
    // ÉTAPE 3: Créer transformations
    let transforms: Vec<Box<dyn Transform>> = config
        .transforms
        .iter()
        .map(|t| create_transform(t))  // ← NOLACK le fera
        .collect();
    
    // ÉTAPE 4: Créer écrivain
    let mut writer = create_writer(&config.destination);
    
    // ÉTAPE 5: Boucle principale
    for result in reader.records() {
        Ok(record) => {
            // Envelopper dans Option pour gérer les filtres
            let mut record_option: Option<_> = Some(record);
            
            // Appliquer chaque transformation
            for transform in &transforms {
                record_option = match record_option {
                    Some(rec) => transform.apply(rec),
                    None => None,
                };
                if record_option.is_none() {
                    break;
                }
            }
            
            // Écrire si pas filtré
            if let Some(record) = record_option {
                writer.write_record(&record)?;
                stats.records_written += 1;
            }
        }
    }
    
    // ÉTAPE 6: Finaliser
    writer.finalize()?;
    
    // ÉTAPE 7: Rapport
    stats.stop();
    stats.print_report();
    
    Ok(())
}
```

### Factories (tes usines)

```rust
// Factory des lecteurs
fn create_reader(config: &SourceConfig) -> Result<Box<dyn SourceReader>> {
    match config.format.to_lowercase().as_str() {
        "csv" => Ok(Box::new(CsvReader { ... })),
        "json" => Ok(Box::new(JsonReader { ... })),
        "delimited" => Ok(Box::new(DelimitedReader { ... })),
        _ => Err(anyhow!("Format non supporté"))
    }
}
```

**Pourquoi `Box<dyn Trait>` ?**
- `Box` = enveloppe la valeur en mémoire dynamique
- `dyn Trait` = on ne sait pas le type exact à la compilation (CSV/JSON/etc.)
- À l'exécution, Rust envoie sur la bonne implémentation

---

## 🧪 Dépendances entre équipes

### Ce que TU attendes des autres

```
TSEFACK::pipeline.rs
    ├─→ Attend CsvReader de NZEUTEM
    ├─→ Attend JsonReader + DelimitedReader de DIOM
    ├─→ Attend create_transform() de NOLACK
    │   ├─→ Qui utilise RenameTransform (ASSONGUE)
    │   ├─→ Qui utilise FilterTransform (ASSONGUE)
    │   ├─→ Qui utilise CastTransform (NOLACK)
    │   ├─→ Qui utilise ComputeTransform (NOLACK)
    │   └─→ Qui utilise DropTransform (NOLACK)
    ├─→ Attend create_writer() de NGANSOP
    │   ├─→ Qui utilise CsvWriter (NGLITANG)
    │   ├─→ Qui utilise JsonWriter (NGLITANG)
    │   └─→ Qui utilise JsonlWriter (NGANSOP)
    └─→ Attend stats.print_report() de DONFACK
```

### Ce que les autres attendent de toi

**Rien !** 😄 Tu es indépendant.

Mais les autres ont besoin que tu aies:
- ✅ Défini les traits: `SourceReader`, `Transform`, `SinkWriter`
- ✅ Montré comment appeler leurs fonctions

**C'est déjà fait !**

---

## 🔍 Point de contrôle : Comment tester que c'est bon ?

Quand une équipe te dit "j'ai fini", tu fais :

```bash
# 1. Recompile
cargo build

# 2. Lance un test simple
./target/debug/datapipe --config test_pipeline.toml

# 3. Vérifier le rapport
# Doit afficher le nombre de records traités ✅
```

---

## 💡 Patterns Rust clés dans ton code

### Pattern 1: `Result` pour les erreurs
```rust
pub fn run(config_path: &Path) -> Result<()> {
    let config = PipelineConfig::from_file(config_path)?;  // ← ? = "remonte l'erreur"
    // ...
    Ok(())  // ← Succès
}
```
**Signification** : Si une erreur dans `from_file()`, on quitte avec l'erreur. Sinon on continue.

### Pattern 2: `Box<dyn Trait>` pour la polymorphie
```rust
fn create_reader(config: &SourceConfig) -> Result<Box<dyn SourceReader>> {
    match config.format {
        "csv" => Ok(Box::new(CsvReader { ... })),
        "json" => Ok(Box::new(JsonReader { ... })),
        // ...
    }
}
```
**Signification** : On retourne une "boîte" qui pourrait contenir N'IMPORTE QUEL type implémentant `SourceReader`. Le vrai type est décidé à l'exécution.

### Pattern 3: Option<T> pour les valeurs optionnelles
```rust
let mut record_option: Option<_> = Some(record);

for transform in &transforms {
    record_option = match record_option {
        Some(rec) => transform.apply(rec),
        None => None,
    };
}

if let Some(record) = record_option {
    writer.write_record(&record)?;
}
```
**Signification** : `None` = le record a été filtré. `Some(record)` = on peut l'écrire.

---

## ✅ Checklist : Qu'est-ce que tu as terminé ?

- [x] **Cargo.toml** - Dépendances en place
- [x] **main.rs** - CLI parsing (appelle pipeline::run)
- [x] **config.rs** - Structures TOML (fourni par l'équipe)
- [x] **pipeline.rs** - Orchestrateur complet ⭐
  - [x] Charge la config
  - [x] Factory de lecteurs
  - [x] Intègre les transformations
  - [x] Crée l'écrivain
  - [x] Boucle de traitement
  - [x] Rapport final
- [x] **README.md** - Documentation complète

---

## 🚀 Prochaines étapes

Maintenant que **ta part est finie**, tu as 2 options:

### Option 1: Aider les autres (Recommandé!)
- [ ] Vérifier que chaque équipe a bien compris leur trait
- [ ] Tester leur code avec tes factories
- [ ] Donner du feedback sur la qualité

### Option 2: Implémenter les bonus (Advanced)
- [ ] Mode `--dry-run` (simule sans écrire)
- [ ] Mode `--watch` (relance à chaque modif)
- [ ] Rapport HTML (DONFACK)
- [ ] Jointure entre fichiers (ATEKOUMBO)

---

## 📞 Contact

Si tu as des questions sur la coordination:
- Demande aux personnes de tester avec toi
- Montre-leur comment appeler `pipeline::run()`
- Valide que tout compile ensemble

**Bon courage! Tu as fait du super travail.** 🎉
