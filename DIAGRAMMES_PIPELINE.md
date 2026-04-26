# 🎨 Visualisation du Pipeline TSEFACK

## Diagramme 1 : Flux de données complet

```
                    USER LAUNCHES
                    datapipe --config pipeline.toml
                           ↓
                   ┌────────────────┐
                   │   CLI Parse    │ main.rs
                   │ (clap crate)   │
                   └────────┬───────┘
                            ↓
                   ┌────────────────┐
                   │  load_config   │ config.rs
                   │ pipeline.toml  │
                   └────────┬───────┘
                            ↓
                ╔═══════════════════════════╗
                ║                           ║
                ║  pipeline.rs::run()       ║  ← TSEFACK ORCHESTRATES HERE
                ║  (ÉTAPE 1-7)              ║
                ║                           ║
                ║  ┌──────────────────┐     ║
                ║  │ create_reader    │     ║
                ║  ├─→ CSV Reader     │     ║ Lecteurs implémentés
                ║  ├─→ JSON Reader    │     ║ par NZEUTEM, DIOM
                ║  └─→ Delim Reader   │     ║
                ║  └──────────────────┘     ║
                ║           ↓               ║
                ║  ┌──────────────────┐     ║
                ║  │ create_transform │     ║ Transformations par
                ║  │ (x N records)    │     ║ ASSONGUE, NOLACK
                ║  └──────────────────┘     ║
                ║           ↓               ║
                ║  ┌──────────────────┐     ║
                ║  │ create_writer    │     ║ Écrivains par
                ║  │ CSV/JSON/JSONL   │     ║ NGLITANG, NGANSOP
                ║  └──────────────────┘     ║
                ║           ↓               ║
                ║  ╔════════════════════╗   ║
                ║  ║  FOR each record:  ║   ║ ← BOUCLE PRINCIPALE
                ║  ║  ├─ read()         ║   ║
                ║  ║  ├─ transform()    ║   ║
                ║  ║  ├─ filter()       ║   ║
                ║  ║  └─ write()        ║   ║
                ║  ╚════════════════════╝   ║
                ║           ↓               ║
                ║  ┌──────────────────┐     ║
                ║  │ print_report()   │     ║ Stats par DONFACK
                ║  │ records/errors   │     ║
                ║  └──────────────────┘     ║
                ║                           ║
                ╚═══════════════════════════╝
                            ↓
                   ┌────────────────┐
                   │ OUTPUT FILE    │
                   │ (CSV/JSON)     │
                   └────────────────┘
```

---

## Diagramme 2 : Les 7 étapes du pipeline TSEFACK

```
ÉTAPE 1: LOAD CONFIG
   pipeline.toml
   ├─ source: format, path, delimiter
   ├─ destination: format, path  
   └─ transforms: [...]
          ↓
ÉTAPE 2: CREATE READER
   Source Config
   ├─ "csv" → CsvReader(path, delimiter)
   ├─ "json" → JsonReader(path)
   └─ "delimited" → DelimitedReader(path, delimiter)
          ↓
ÉTAPE 3: CREATE TRANSFORMS
   Transforms Config
   ├─ "rename" → RenameTransform(from, to)
   ├─ "filter" → FilterTransform(column, value, op)
   ├─ "cast" → CastTransform(column, type)
   ├─ "compute" → ComputeTransform(column, expr)
   └─ "drop" → DropTransform(column)
          ↓
ÉTAPE 4: CREATE WRITER
   Destination Config
   ├─ "csv" → CsvWriter(path)
   ├─ "json" → JsonWriter(path)
   └─ "jsonl" → JsonlWriter(path)
          ↓
ÉTAPE 5: MAIN LOOP
   For each record from reader:
      │
      ├─ Envelopper: Option<Record> = Some(record)
      │
      ├─ FOR each transform:
      │  ├─ apply(record)
      │  ├─ if Some(new_record) → continue
      │  └─ if None → FILTRÉ, break
      │
      └─ IF record not filtered:
         └─ writer.write_record()
          ↓
ÉTAPE 6: FINALIZE
   writer.finalize()  // Flush buffers
          ↓
ÉTAPE 7: REPORT
   stats.print_report()
   ├─ records_read: 1000
   ├─ records_written: 850
   ├─ records_filtered: 150
   ├─ errors: 0
   └─ duration: 45ms
```

---

## Diagramme 3 : Dépendances entre modules

```
                        main.rs
                          ↓
                     pipeline.rs ← TSEFACK
                    /    |    \
                   /     |     \
            reader/   transform/  writer/
           /          |          \
      csv_reader    factory    csv_writer
      json_reader  /  |  \    json_writer
      delim_read  rename  cast  jsonl_writer
                  filter  compute
                  drop    factory

                        config.rs
                          ↑
            (parsed into all modules)

                        stats.rs
                          ↑
                 (updated by pipeline.rs)
```

---

## Diagramme 4 : Exemple concret - Flux d'un record

```
CSV FILE:
┌────────────────────────────────┐
│ nom_complet │ dept    │ salary │
├────────────────────────────────┤
│ Alice M.    │ IT      │ 5000   │
└────────────────────────────────┘
        ↓
   READER.RECORDS()
        ↓
 Record = {
   "nom_complet": "Alice M.",
   "dept": "IT",
   "salary": "5000"
 }
        ↓
   OPTION<RECORD> = SOME(record)
        ↓
   TRANSFORM 1: RENAME
   ├─ Match: Some(rec)
   ├─ Apply: rec.remove("nom_complet")
   │         rec.insert("nom", "Alice M.")
   └─ Result: Some({...})
        ↓
   TRANSFORM 2: FILTER
   ├─ Match: Some(rec)
   ├─ Check: dept == "IT" ? YES
   └─ Result: Some({...})
        ↓
   TRANSFORM 3: DROP
   ├─ Match: Some(rec)
   ├─ Apply: rec.remove("salary")
   └─ Result: Some({...})
        ↓
   WRITER.WRITE_RECORD()
   Envoyer: {
     "nom": "Alice M.",
     "dept": "IT"
   }
        ↓
   JSON FILE:
   [{
     "nom": "Alice M.",
     "dept": "IT"
   }]
```

---

## Diagramme 5 : Gestion des erreurs au runtime

```
pipeline.rs::run()
├─ PipelineConfig::from_file()?
│  └─ ❌ File not found
│     └─ Err → main retourne erreur
│
├─ create_reader()?
│  └─ ❌ Format inconnu
│     └─ Err → main retourne erreur
│
├─ create_transform()
│  └─ ❌ Transform type inconnu
│     └─ Err → log warning, continue
│
├─ FOR record in reader.records()
│  ├─ Ok(record) → traiter
│  └─ Err(e) → log error, stats.errors_encountered++
│
├─ writer.write_record()?
│  └─ ❌ Can't write
│     └─ log error, continue
│
└─ writer.finalize()?
   └─ ❌ Last write failed
      └─ Err → main retourne erreur
```

---

## Diagramme 6 : État du projet

```
TSEFACK PART (100% DONE) ✅
├─ [x] main.rs - CLI interface
├─ [x] config.rs - TOML structures  
├─ [x] pipeline.rs - Orchestrateur
│   ├─ [x] Load config
│   ├─ [x] Reader factory
│   ├─ [x] Transform handling
│   ├─ [x] Writer creation
│   ├─ [x] Main loop
│   ├─ [x] Finalize
│   └─ [x] Report
├─ [x] README.md - User docs
└─ [x] GUIDE_TSEFACK.md - Your docs

BLOCKING ON OTHER TEAMS:
├─ [ ] NZEUTEM - CSV Reader implementation
├─ [ ] DIOM - JSON/Delimited readers
├─ [ ] ASSONGUE - Rename/Filter transforms
├─ [ ] NOLACK - Cast/Compute/Drop + Factory
├─ [ ] NGLITANG - CSV/JSON writers
├─ [ ] NGANSOP - JSONL writer + factory
├─ [ ] DONFACK - Stats/Report formatting
├─ [ ] ATEKOUMBO - Join functionality
└─ [ ] NJOH - Watch/Dry-run modes
```

---

## Résumé: Le rôle d'orchestrateur

| Aspect | Toi (TSEFACK) | Les autres |
|--------|---|---|
| Interfaces | Définis les traits | Les implémentent |
| Logique | Assembles tout | Font des briques |
| Erreurs | Gères au niveau pipeline | Gèrent au niveau composant |
| Testing | Tests d'intégration | Tests unitaires |
| Documentation | Vue générale | Détails spécifiques |

**Conclusion:** Tu n'écris pas toutes les briques, mais tu **orchestres** comment elles s'assemblent. C'est comme être chef d'orchestre! 🎼🎺🎸🥁

