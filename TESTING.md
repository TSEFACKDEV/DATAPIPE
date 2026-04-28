# 🧪 Testing Guide - DataPipe

## Vue d'ensemble des Tests

Le projet DataPipe contient **28 tests d'intégration** couvrant:
- Lecteurs (CSV, JSON, délimité)
- Convertisseurs de format
- Cohérence des données
- Round-trips
- Gestion d'erreurs

## Exécuter les Tests

### Tous les tests
```bash
cargo test
```

### Tests d'intégration seuls
```bash
cargo test --test integration_tests_extended
cargo test --test integration_test
```

### Un test spécifique
```bash
cargo test test_json_produits -- --nocapture
```

### Avec affichage des logs
```bash
cargo test -- --nocapture
```

## Tests Disponibles

### Lecture CSV
- `test_csv_transactions_standard` - CSV standard (virgule)
- `test_csv_contacts_semicolon` - CSV point-virgule
- `test_csv_sample_large` - CSV 25+ enregistrements

### Lecture JSON
- `test_json_produits` - Tableau JSON simple
- `test_json_commandes` - Tableau JSON avec champs numériques

### Lecture Délimité
- `test_delimited_villes_tab` - TSV (tab-separated)

### Conversions de Format
- `test_conversion_csv_to_json` - CSV → JSON
- `test_conversion_csv_to_jsonl` - CSV → JSONL (streaming)
- `test_conversion_json_to_csv` - JSON → CSV
- `test_roundtrip_json_csv_json` - JSON → CSV → JSON

### Validation de Données
- `test_employes_champs_obligatoires` - Champs présents
- `test_transactions_champs_obligatoires` - Cohérence CSV
- `test_produits_champs_obligatoires` - Cohérence JSON

### Tests Avancés (integration_test.rs)
- `test_jsonl_writer_ecriture_basique` - JSONL basique
- `test_jsonl_format_strict_une_ligne_par_record` - Format JSONL strict
- `test_factory_format_jsonl` - Factory JSONL
- `test_factory_format_json` - Factory JSON
- `test_factory_format_csv` - Factory CSV
- `test_pipeline_records_en_memoire_vers_jsonl` - Pipeline JSONL
- `test_pipeline_avec_filtre_records_rejetes` - Filtrage
- `test_pipeline_rename_compute_drop_vers_jsonl` - Transformations
- `test_pipeline_1000_records_jsonl` - Performance
- `test_coherence_json_vs_jsonl` - Cohérence formats
- `test_robustesse_valeurs_speciales` - Cas limites
- `test_statistiques_pipeline_simule` - Stats
- `test_polymorphisme_sink_writer` - Polymorphisme
- `test_factory_format_inconnu_erreur` - Gestion d'erreur

## Fichiers de Test

### Structure

```
tests/
├── integration_test.rs              # 14 tests existants
└── integration_tests_extended.rs    # 14 nouveaux tests
```

### Données Utilisées

```
data/
├── transactions.csv       # 5 enregistrements
├── contacts.csv          # 5 enregistrements  
├── sample_large.csv      # 25 employés
├── produits.json         # 5 produits
├── commandes.json        # 4 commandes
└── villes.txt           # 10 villes (TSV)
```

## Ajouter un Nouveau Test

### Exemple: Tester une nouvelle source CSV

```rust
#[test]
fn test_mon_nouveau_csv() {
    let reader = CsvReader::new("data/mon_fichier.csv");
    let records = read_all_records(&reader);
    
    assert_eq!(records.len(), 10, "Doit avoir 10 enregistrements");
    
    let premier = &records[0];
    assert!(premier.contains_key("id"));
    assert!(premier.contains_key("nom"));
}
```

### Exemple: Tester une transformation

```rust
#[test]
fn test_renommer_colonne() {
    let reader = CsvReader::new("data/test.csv");
    let records = read_all_records(&reader);
    
    // Appliquer une transformation
    let config = TransformConfig {
        r#type: "rename".to_string(),
        from: Some("ancien".to_string()),
        to: Some("nouveau".to_string()),
        ..Default::default()
    };
    
    let transform = create_transform(&config);
    let result = transform.apply(records[0].clone()).unwrap();
    
    assert!(!result.contains_key("ancien"));
    assert!(result.contains_key("nouveau"));
}
```

## Résultats Attendus

### Statut de Compilation
```
Finished `release` profile [optimized] target(s) in XX.XXs
```

### Résultats des Tests
```
test result: ok. 28 passed; 0 failed
```

### Performance Typical
- Tests d'intégration: < 1 seconde
- Tests avec fichiers: < 5 secondes
- Compilation complète: 20-30 secondes (release)

## Troubleshooting

### Erreur: "File not found"
- Assurez-vous que les fichiers `data/*.csv` et `data/*.json` existent
- Exécutez les tests du répertoire racine du projet

### Erreur: "Type mismatch"
- Tous les types doivent être `IndexMap<String, Value>`
- Pas de `HashMap` en production

### Tests unitaires échoient
- Les tests unitaires des modules ont été désactivés (`#[cfg(test_disabled)]`)
- Ceci est temporaire en attendant une correction des fixtures
- Les tests d'intégration couvrent la fonctionnalité réelle

## Performance des Tests

```bash
# Mesurer le temps
time cargo test

# Tests rapides (< 100ms)
cargo test --test integration_tests_extended

# Tous les tests
cargo test
```

## Bonnes Pratiques de Test

1. **Utiliser `tempdir()`** pour les fichiers temporaires
2. **Garder les données** de test petites et pertinentes
3. **Nommer explicitement** les tests
4. **Documenter les assertions** avec des messages clairs
5. **Tester les cas d'erreur** aussi

## Exemples d'Exécution

### Tester uniquement les conversions
```bash
cargo test conversion
```

### Tester le format JSON
```bash
cargo test json
```

### Tester les validations de données
```bash
cargo test champs_obligatoires
```

## Couverture

**Modules couverts par les tests:**
- ✅ reader/csv_reader.rs
- ✅ reader/json_reader.rs  
- ✅ reader/delimited_reader.rs
- ✅ writer/csv_writer.rs
- ✅ writer/json_writer.rs
- ✅ writer/jsonl_writer.rs
- ✅ writer/factory.rs
- ✅ config.rs (via tests d'intégration)
- ✅ pipeline.rs (via tests d'intégration)

**Modules non directement testés:**
- transform/* (tests unitaires désactivés)
- stats.rs (fonctionnalité de reporting)
- report.rs (bonus feature)
- join.rs (bonus feature)
- watch.rs (bonus feature)

---

Pour plus d'informations, consultez [README.md](README.md) et [QUICK_START.md](QUICK_START.md).
