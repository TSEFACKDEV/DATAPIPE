# DataPipe - Outil ETL en Rust

DataPipe est un outil ETL (Extract, Transform, Load) développé en Rust dans le cadre du cours de Programmation Système à l'ENSPD.

##  Installation

### Prérequis
- Rust 1.75.0 ou supérieur
- Cargo

### Compilation
```bash
cargo build --release

📖 Utilisation

Mode normal
bash
datapipe --config pipeline.toml
Mode dry-run (simulation)
bash
datapipe --config pipeline.toml --dry-run
Mode watch (surveillance)
bash
datapipe --config pipeline.toml --watch --interval 30  


📁 Structure du projet

text
datapipe/
├── src/
│   ├── reader/      # Lecteurs de formats (CSV, JSON, texte)
│   ├── transform/   # Transformations (rename, filter, cast, etc.)
│   ├── writer/      # Écrivains (CSV, JSON, JSONL)
│   ├── main.rs      # Point d'entrée CLI
│   ├── config.rs    # Configuration TOML
│   └── ...
├── examples/        # Exemples de pipelines
├── tests/           # Tests d'intégration
└── data/            # Données de test


👥 Équipe

Chef de projet : TSEFACK CALVIN KLEIN

Membres : NZEUTEM, DIOM, ASSONGUE, NOLACK, NGLITANG, NGANSOP, DONFACK, ATEKOUMBO, NJOH

