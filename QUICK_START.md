# ⚡ Quick Start - DataPipe en 5 minutes

## 1️⃣ Compiler

```bash
cargo build --release
```

L'exécutable se trouvera dans `target/release/datapipe`

## 2️⃣ Créer un fichier de configuration

Créez `pipeline.toml`:

```toml
[source]
format = "csv"
path = "data/transactions.csv"
delimiter = ","

[destination]
format = "json"
path = "output/resultat.json"

[[transforms]]
type = "filter"
column = "statut"
value = "complete"
operator = "="

[[transforms]]
type = "drop"
column = "email"
```

## 3️⃣ Exécuter le pipeline

```bash
./target/release/datapipe --config pipeline.toml
```

Le fichier `output/resultat.json` contient vos données transformées! 🎉

## 📚 Exemples Prêts à l'Emploi

### Transformation simple (CSV → JSON)
```bash
./target/release/datapipe --config examples/employes_transform.toml
```

### Streaming (CSV → JSONL)
```bash
./target/release/datapipe --config examples/transactions_stream.toml
```

### Filtrage avancé (JSON → CSV)
```bash
./target/release/datapipe --config examples/produits_promo.toml
```

### Délimité personnalisé
```bash
./target/release/datapipe --config examples/villes_grandes.toml
```

## 🔍 Mode Dry-Run (aperçu sans écriture)

```bash
./target/release/datapipe --config pipeline.toml --dry-run
```

## 🧪 Tests

```bash
# Tests d'intégration
cargo test --test integration_tests_extended

# Tests complets
cargo test
```

## 🔄 Transformations Disponibles

### rename
```toml
[[transforms]]
type = "rename"
from = "ancien_nom"
to = "nouveau_nom"
```

### filter
```toml
[[transforms]]
type = "filter"
column = "age"
value = "30"
operator = ">"  # =, !=, <, <=, >, >=, contains
```

### drop
```toml
[[transforms]]
type = "drop"
column = "secret"
```

### cast
```toml
[[transforms]]
type = "cast"
column = "montant"
target_type = "number"  # string, number, boolean
```

### compute
```toml
[[transforms]]
type = "compute"
new_column = "double_montant"
expression = "montant * 2"
```

## 📂 Structure des Données

### CSV avec délimiteur virgule
```csv
nom,age,ville
Alice,25,Paris
Bob,30,Lyon
```

### JSON tableau
```json
[
  {"nom": "Alice", "age": 25},
  {"nom": "Bob", "age": 30}
]
```

### Délimité personnalisé (ex: tab)
```
nom	age	ville
Alice	25	Paris
Bob	30	Lyon
```

## ✨ Résultat

Tous les formats de sortie sont supportés:
- **CSV**: Compatible Excel
- **JSON**: Tableau complet
- **JSONL**: Une ligne = un objet (idéal pour gros volumes)

---

**Besoin d'aide?** Consulter [README.md](README.md) pour la documentation complète.
