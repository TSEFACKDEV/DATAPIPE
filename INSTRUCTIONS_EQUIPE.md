# 📋 GUIDE DE TRAVAIL - DATAPIPE GROUPE 6

## 🎯 Objectif Global
Créer un outil ETL (Extract, Transform, Load) en Rust capable de :
- Lire des fichiers CSV, JSON, texte délimité
- Appliquer des transformations (rename, filter, cast, compute, drop)
- Écrire en CSV, JSON, JSONL
- Fonctionnalités bonus : JOIN, rapport HTML, mode watch, dry-run

---

## 📁 STRUCTURE DU PROJET
datapipe/
├── Cargo.toml # [TSEFACK] Dépendances du projet
├── README.md # [TSEFACK] Documentation utilisateur
├── GUIDE_TRAVAIL_EQUIPE.md # [TSEFACK] Ce fichier
├── pipeline.toml # [TSEFACK] Exemple de configuration
│
├── src/
│ ├── main.rs # [TSEFACK] Point d'entrée CLI
│ ├── config.rs # [TSEFACK] Structures de configuration TOML
│ ├── pipeline.rs # [TSEFACK] Orchestrateur principal
│ │
│ ├── reader/
│ │ ├── mod.rs # [NZEUTEM] Trait SourceReader + type Record
│ │ ├── csv_reader.rs # [NZEUTEM] Lecteur CSV
│ │ ├── json_reader.rs # [DIOM] Lecteur JSON
│ │ └── delimited_reader.rs # [DIOM] Lecteur texte délimité
│ │
│ ├── transform/
│ │ ├── mod.rs # [ASSONGUE] Trait Transform
│ │ ├── rename.rs # [ASSONGUE] Renommage de colonnes
│ │ ├── filter.rs # [ASSONGUE] Filtrage par valeur
│ │ ├── cast.rs # [NOLACK] Changement de type
│ │ ├── compute.rs # [NOLACK] Calcul de nouvelles colonnes
│ │ ├── drop.rs # [NOLACK] Suppression de colonnes
│ │ └── factory.rs # [NOLACK] Fabrique de transformations
│ │
│ ├── writer/
│ │ ├── mod.rs # [NGLITANG] Trait SinkWriter
│ │ ├── csv_writer.rs # [NGLITANG] Écrivain CSV
│ │ ├── json_writer.rs # [NGLITANG] Écrivain JSON
│ │ ├── jsonl_writer.rs # [NGANSOP] Écrivain JSONL
│ │ └── factory.rs # [NGANSOP] Fabrique d'écrivains
│ │
│ ├── stats.rs # [DONFACK] Statistiques d'exécution
│ ├── validation.rs # [DONFACK] Validation de schéma
│ ├── report.rs # [DONFACK] Génération rapport HTML
│ ├── join.rs # [ATEKOUMBO] Jointure entre fichiers
│ └── watch.rs # [NJOH] Mode surveillance
│
├── examples/
│ ├── basic/ # [NJOH] Exemple simple CSV → JSON
│ ├── join_demo/ # [ATEKOUMBO] Exemple jointure
│ └── hospital/ # [NJOH] Cas usage hôpital
│
├── tests/
│ └── integration_test.rs # [NGANSOP] Tests d'intégration
│
└── data/ # [TOUS] Données de test
├── test.csv
├── test.json
└── test_delimited.txt

text

---

## 👥 RÉPARTITION DES TÂCHES PAR PERSONNE

---

### #01 - TSEFACK CALVIN KLEIN (Chef de Projet)

#### 📝 Fichiers à travailler :
- `Cargo.toml` - Configuration du projet et dépendances
- `src/main.rs` - Point d'entrée avec CLI (clap)
- `src/config.rs` - Structures de configuration TOML
- `src/pipeline.rs` - Orchestrateur principal (À CRÉER)
- `README.md` - Documentation du projet
- `GUIDE_TRAVAIL_EQUIPE.md` - Guide pour l'équipe

#### 🔧 Ce que tu dois implémenter :

**1. Cargo.toml**
```rust
// TODO: Vérifier que toutes les dépendances sont là
// - csv, serde, serde_json, toml, clap, anyhow, indexmap
2. src/main.rs

rust
// TODO: Compléter la CLI avec clap
// - Argument --config pour le fichier TOML
// - Flag --dry-run pour la simulation
// - Flag --watch pour la surveillance
// - Appeler run_pipeline() avec la config
3. src/config.rs

rust
// TODO: Compléter les structures de désérialisation TOML
// - PipelineConfig (source, destination, transforms)
// - SourceConfig (format, path, delimiter)
// - DestinationConfig (format, path)
// - TransformConfig (type, from, to, column, etc.)
// - JoinConfig (pour le bonus)
// - Implémenter PipelineConfig::from_file()
4. src/pipeline.rs (À CRÉER)

rust
// TODO: Implémenter l'orchestrateur principal
// 1. Charger la config TOML
// 2. Créer le lecteur selon source.format
// 3. Créer les transformations selon transforms[]
// 4. Créer l'écrivain selon destination.format
// 5. Pour chaque record:
//    a. Appliquer chaque transformation
//    b. Si Some(record), écrire
//    c. Si None, incrémenter records_filtered
// 6. Finaliser l'écriture
// 7. Afficher les statistiques
🎯 Utilité dans le projet :
Tu es le chef d'orchestre. Ton code assemble toutes les pièces créées par l'équipe :

Les lecteurs (NZEUTEM, DIOM)

Les transformations (ASSONGUE, NOLACK)

Les écrivains (NGLITANG, NGANSOP)

Les stats (DONFACK)

Les bonus (ATEKOUMBO, NJOH)

Sans toi, chaque partie fonctionne seule mais pas ensemble !

#02 - NZEUTEM DOMMOE EUNICE FELIXTINE (Lecteur CSV)
📝 Fichiers à travailler :
src/reader/mod.rs - Trait SourceReader + type Record

src/reader/csv_reader.rs - Implémentation du lecteur CSV

data/test.csv - Fichier CSV de test (à créer)

tests/csv_test.rs - Tests unitaires (dans le même fichier)

🔧 Ce que tu dois implémenter :
1. src/reader/mod.rs

rust
// TODO: Définir le type Record = HashMap<String, Value>
// TODO: Définir le trait SourceReader avec méthode records()
2. src/reader/csv_reader.rs

rust
// TODO: Implémenter CsvReader avec:
// - Champs: path (String), delimiter (u8)
// - Méthode records() qui:
//   1. Ouvre le fichier CSV avec csv::Reader
//   2. Lit la première ligne comme en-têtes
//   3. Pour chaque ligne, crée un Record (HashMap)
//   4. Retourne un itérateur Box<dyn Iterator<Item = Result<Record>>>
3. Tests

rust
// TODO: Écrire les tests unitaires
// - Test avec un petit CSV (2-3 lignes)
// - Vérifier que les colonnes sont bien lues
// - Vérifier que les valeurs sont correctes
// - Test avec délimiteur point-virgule
🎯 Utilité dans le projet :
Ton lecteur CSV est la porte d'entrée pour tous les fichiers Excel/CSV.
C'est le format le plus courant en entreprise. Sans toi, DataPipe ne peut pas lire les fichiers des PME, hôpitaux, ministères qui utilisent tous Excel.

#03 - DIOM LUCRAINE LETHICIA FIEN (Lecteur JSON & Texte)
📝 Fichiers à travailler :
src/reader/json_reader.rs - Implémentation du lecteur JSON

src/reader/delimited_reader.rs - Implémentation du lecteur texte délimité

data/test.json - Fichier JSON de test (à créer)

data/test_delimited.txt - Fichier texte de test (à créer)

🔧 Ce que tu dois implémenter :
1. src/reader/json_reader.rs

rust
// TODO: Implémenter JsonReader avec:
// - Champ: path (String)
// - Méthode records() qui:
//   1. Ouvre le fichier JSON
//   2. Parse un tableau d'objets [{}, {}, {}]
//   3. Convertit chaque objet en Record (HashMap)
//   4. Retourne un itérateur
2. src/reader/delimited_reader.rs

rust
// TODO: Implémenter DelimitedReader avec:
// - Champs: path (String), delimiter (u8)
// - Même logique que CSV mais avec délimiteur personnalisable
// - Gérer tabulation (\t), point-virgule (;), pipe (|)
3. Tests

rust
// TODO: Écrire les tests unitaires
// - Test JSON avec un tableau de 2-3 objets
// - Test délimité avec tabulations
// - Test délimité avec point-virgule
🎯 Utilité dans le projet :
Tu gères les formats modernes (JSON des APIs web) et les formats legacy (texte délimité). Les applications mobiles envoient du JSON, les vieux systèmes exportent en texte délimité. Tu rends DataPipe compatible avec tous ces formats.

#04 - ASSONGUE TATANG MURIEL (Transformations Rename & Filter)
📝 Fichiers à travailler :
src/transform/mod.rs - Trait Transform

src/transform/rename.rs - Transformation Rename

src/transform/filter.rs - Transformation Filter

🔧 Ce que tu dois implémenter :
1. src/transform/mod.rs

rust
// TODO: Définir le trait Transform avec:
// - apply(&self, record: Record) -> Option<Record>
// - name(&self) -> &str
2. src/transform/rename.rs

rust
// TODO: Implémenter RenameTransform avec:
// - Champs: from (String), to (String)
// - apply(): 
//   1. Vérifier si la colonne 'from' existe
//   2. Supprimer l'ancienne clé avec remove()
//   3. Insérer la valeur avec le nouveau nom
//   4. Retourner Some(record)
3. src/transform/filter.rs

rust
// TODO: Implémenter FilterTransform avec:
// - Champs: column (String), value (String), operator (String)
// - apply():
//   1. Récupérer la valeur de la colonne
//   2. Comparer selon l'opérateur (=, !=, <, >)
//   3. Si condition vraie → Some(record)
//   4. Si condition fausse → None (record filtré!)
4. Tests

rust
// TODO: Écrire les tests unitaires
// - Test rename: vérifier que la colonne est renommée
// - Test rename colonne inexistante: ne doit pas planter
// - Test filter égalité: garder si valeur correspond
// - Test filter inégalité: filtrer si valeur différente
// - Test filter numérique: < et >
🎯 Utilité dans le projet :
Tu crées le standard de transformation. Toutes les autres transformations suivront ton trait Transform.

Rename : essentiel car chaque source nomme ses colonnes différemment

Filter : permet de nettoyer les données (ex: garder seulement les patients hospitalisés)

#05 - NOLACK KAWUNJIBI FRANGE PARKER (Transformations Cast, Compute, Drop)
📝 Fichiers à travailler :
src/transform/cast.rs - Transformation Cast

src/transform/compute.rs - Transformation Compute

src/transform/drop.rs - Transformation Drop

src/transform/factory.rs - Fabrique de transformations

🔧 Ce que tu dois implémenter :
1. src/transform/cast.rs

rust
// TODO: Implémenter CastTransform avec:
// - Champs: column (String), target_type (String)
// - apply():
//   1. Récupérer la valeur actuelle
//   2. Convertir String → Number (parse::<f64>())
//   3. Convertir String → Boolean ("true"/"false")
//   4. Convertir Number → String (to_string())
//   5. Mettre à jour le record avec la nouvelle valeur
2. src/transform/compute.rs

rust
// TODO: Implémenter ComputeTransform avec:
// - Champs: new_column (String), expression (String)
// - apply():
//   1. Parser l'expression simple (ex: "salaire * 0.1")
//   2. Remplacer les noms de colonnes par leurs valeurs
//   3. Évaluer l'expression mathématique (+, -, *, /)
//   4. Ajouter la nouvelle colonne au record
//   Version simple: gérer seulement "colonne * nombre"
3. src/transform/drop.rs

rust
// TODO: Implémenter DropTransform avec:
// - Champ: column (String)
// - apply():
//   1. Supprimer la colonne avec remove()
//   2. Retourner Some(record)
//   Même si la colonne n'existe pas, ne pas échouer
4. src/transform/factory.rs

rust
// TODO: Implémenter create_transform() qui:
// - Prend un TransformConfig
// - Match sur config.type:
//   "rename" → RenameTransform
//   "filter" → FilterTransform
//   "cast" → CastTransform
//   "compute" → ComputeTransform
//   "drop" → DropTransform
// - Retourne Box<dyn Transform>
5. Tests

rust
// TODO: Écrire les tests unitaires
// - Test cast string → number
// - Test cast string → boolean
// - Test compute: "salaire * 0.1"
// - Test compute: "age + 5"
// - Test drop colonne existante
// - Test drop colonne inexistante (ne doit pas planter)
// - Test factory: créer chaque type de transformation
🎯 Utilité dans le projet :
Tu ajoutes la puissance de calcul à DataPipe :

Cast : convertit les types (ex: âge en texte → nombre)

Compute : crée de nouvelles colonnes (ex: prime = salaire * 0.1)

Drop : supprime les colonnes sensibles (ex: mot de passe)

Factory : permet de créer n'importe quelle transformation depuis le fichier TOML

#06 - NGLITANG RUBEN (Écrivains CSV & JSON)
📝 Fichiers à travailler :
src/writer/mod.rs - Trait SinkWriter

src/writer/csv_writer.rs - Écrivain CSV

src/writer/json_writer.rs - Écrivain JSON

🔧 Ce que tu dois implémenter :
1. src/writer/mod.rs

rust
// TODO: Définir le trait SinkWriter avec:
// - write_record(&mut self, record: &Record) -> Result<()>
// - finalize(&mut self) -> Result<()>
2. src/writer/csv_writer.rs

rust
// TODO: Implémenter CsvSinkWriter avec:
// - Champs: writer (Writer<BufWriter<File>>), headers_written (bool), headers (Vec<String>)
// - new(path, headers): constructeur
// - write_record():
//   1. Si première écriture, écrire les en-têtes
//   2. Pour chaque en-tête, récupérer la valeur du record
//   3. Écrire la ligne CSV
// - finalize(): flush le writer
3. src/writer/json_writer.rs

rust
// TODO: Implémenter JsonSinkWriter avec:
// - Champs: records (Vec<Record>), output_path (String)
// - write_record(): accumuler les records dans le Vec
// - finalize():
//   1. Ouvrir le fichier de sortie
//   2. Sérialiser tout le Vec en JSON avec serde_json::to_writer_pretty
//   3. Écrire dans le fichier
4. Tests

rust
// TODO: Écrire les tests unitaires
// - Test CSV: écrire 2-3 records, vérifier le fichier
// - Test CSV avec en-têtes
// - Test JSON: écrire 2-3 records, vérifier le fichier JSON
// - Test intégration simple: CSV → JSON
🎯 Utilité dans le projet :
Tu crées la sortie de DataPipe. C'est le résultat final que l'utilisateur voit :

CSV : pour Excel, compatible avec tous les tableurs

JSON : pour les APIs web, les applications modernes

#07 - NGANSOP NGOUABOU FREDI LOIK (Écrivain JSONL & Tests Intégration)
📝 Fichiers à travailler :
src/writer/jsonl_writer.rs - Écrivain JSONL (streaming)

src/writer/factory.rs - Fabrique d'écrivains

tests/integration_test.rs - Tests d'intégration complets

🔧 Ce que tu dois implémenter :
1. src/writer/jsonl_writer.rs

rust
// TODO: Implémenter JsonLinesSinkWriter avec:
// - Champ: writer (BufWriter<File>)
// - write_record():
//   1. Sérialiser le record en JSON sur UNE ligne
//   2. Écrire la ligne avec writeln!
// - finalize(): flush le writer
// AVANTAGE: streaming pur, pas besoin de tout garder en mémoire
2. src/writer/factory.rs

rust
// TODO: Implémenter create_writer() qui:
// - Prend un DestinationConfig
// - Match sur config.format:
//   "csv" → CsvSinkWriter
//   "json" → JsonSinkWriter
//   "jsonl" → JsonLinesSinkWriter
// - Retourne Box<dyn SinkWriter>
3. tests/integration_test.rs

rust
// TODO: Écrire les tests d'intégration
// - Test complet: CSV → transformations → JSON
// - Test complet: CSV → transformations → JSONL
// - Test avec 1000 lignes simulées
// - Test avec filtre (vérifier que certains records sont filtrés)
// - Test avec rename + compute + drop
// - Vérifier les statistiques après exécution
🎯 Utilité dans le projet :
JSONL : format streaming idéal pour les gros fichiers (Big Data)

Factory : permet de choisir le format de sortie depuis le TOML

Tests intégration : garantit que tout fonctionne ensemble

#08 - DONFACK KEUNANG VLADIMIR GEORGES (Stats, Validation, Rapport HTML)
📝 Fichiers à travailler :
src/stats.rs - Statistiques d'exécution

src/validation.rs - Validation de schéma

src/report.rs - Génération rapport HTML

🔧 Ce que tu dois implémenter :
1. src/stats.rs

rust
// TODO: Compléter ExecutionStats avec:
// - records_read: compteur de records lus
// - records_transformed: compteur après transformation
// - records_filtered: compteur de records rejetés
// - records_written: compteur de records écrits
// - errors_encountered: compteur d'erreurs
// - start_time: Instant::now()
// - duration_ms: durée en millisecondes
// - print_report(): afficher un beau rapport dans le terminal
// - stop(): calculer la durée
2. src/validation.rs

rust
// TODO: Implémenter la validation de schéma
// - validate_record(): vérifier les colonnes requises
// - validate_types(): vérifier les types si spécifiés
// - Retourner un Vec<String> avec les erreurs
// - Si pas d'erreur, Vec vide
3. src/report.rs

rust
// TODO: Implémenter generate_html_report()
// - Créer un fichier HTML avec:
//   - Titre "Rapport DataPipe"
//   - Date et heure
//   - Statistiques (lus, filtrés, écrits, durée)
//   - Tableau d'aperçu des 5 premières lignes
//   - Style CSS simple mais propre
//   - Couleurs pour les stats
4. Tests

rust
// TODO: Tests unitaires
// - Test stats: incrémenter les compteurs
// - Test validation: colonnes requises présentes/absentes
// - Test rapport: générer un fichier HTML et vérifier son contenu
🎯 Utilité dans le projet :
Tu rends DataPipe professionnel et fiable :

Stats : l'utilisateur sait exactement ce qui s'est passé

Validation : évite de traiter 1 million de lignes pour échouer à cause d'une faute de frappe

Rapport HTML : livrable propre pour les décideurs

#09 - ATEKOUMBO EXCEL DEXTE JORIS (Jointure & Dry-Run)
📝 Fichiers à travailler :
src/join.rs - Jointure entre fichiers

src/config.rs - Ajouter JoinConfig (coordination avec TSEFACK)

src/main.rs - Ajouter mode --dry-run (coordination avec TSEFACK)

examples/join_demo/ - Exemple de jointure

🔧 Ce que tu dois implémenter :
1. src/join.rs

rust
// TODO: Implémenter la jointure
// - build_lookup():
//   1. Prendre un lecteur SourceReader + nom de clé
//   2. Lire tous les records
//   3. Créer HashMap<String, Record> indexé par la clé
// - join_records():
//   1. Récupérer la valeur de left_key dans le record gauche
//   2. Chercher dans le lookup avec right_key
//   3. Si trouvé: fusionner les deux records (extend)
//   4. Si inner join et pas trouvé: None
//   5. Si left join et pas trouvé: Some(record gauche seul)
2. Mode Dry-Run

rust
// TODO: Dans main.rs, ajouter le mode --dry-run
// - Exécuter tout le pipeline normalement
// - SAUF l'écriture finale dans le fichier
// - Afficher un aperçu des 5 premiers records
// - Afficher les statistiques
// - Message: "[DRY-RUN] Aucun fichier écrit."
3. Exemple join_demo

text
// TODO: Créer un exemple complet
// - commands.csv: commandes avec client_id
// - clients.json: informations clients
// - pipeline.toml: configuration avec [join]
// - Résultat attendu: commandes enrichies avec infos clients
4. Tests

rust
// TODO: Tests unitaires
// - Test inner join: fusionne quand correspondance
// - Test inner join: filtre quand pas de correspondance
// - Test left join: garde même sans correspondance
// - Test dry-run: vérifier qu'aucun fichier n'est créé
🎯 Utilité dans le projet :
JOIN : fonctionnalité puissante comme en SQL, permet de croiser des données de sources différentes

Dry-Run : permet de tester un pipeline sans risque avant de l'exécuter pour de vrai

#10 - NJOH MASSANGO ADOLPHE MACDEAUVILLE (Watch, Documentation, Démo)
📝 Fichiers à travailler :
src/watch.rs - Mode surveillance

examples/basic/ - Exemple simple

examples/hospital/ - Exemple cas d'usage hôpital

data/ - Données de test réalistes (coordination avec tous)

🔧 Ce que tu dois implémenter :
1. src/watch.rs

rust
// TODO: Implémenter le mode watch
// - watch_mode(config_path, interval_secs):
//   1. Lire le fichier source depuis la config
//   2. Obtenir le timestamp de modification
//   3. Boucle infinie:
//      a. Dormir interval_secs secondes
//      b. Vérifier le timestamp actuel
//      c. Si changé: relancer le pipeline
//      d. Mettre à jour le timestamp
2. Exemples

text
// TODO: Créer des exemples fonctionnels
// examples/basic/:
//   - pipeline.toml simple
//   - data.csv avec quelques lignes
//   - Commande: datapipe --config examples/basic/pipeline.toml
//
// examples/hospital/:
//   - Cas réaliste inspiré du document
//   - patients.csv + diagnostics.json
//   - pipeline.toml avec rename, filter, join
3. Données de test

csv
// TODO: Créer data/employees.csv
nom,age,departement,salaire,ville
Jean,25,Informatique,50000,Douala
Marie,30,RH,45000,Yaoundé
Paul,35,Informatique,55000,Bafoussam
4. Documentation

bash
# TODO: Générer la documentation
cargo doc --open
# Vérifier que tous les modules sont documentés
🎯 Utilité dans le projet :
Watch : automatise DataPipe pour les traitements récurrents

Exemples : montre comment utiliser DataPipe concrètement

Démo : c'est toi qui présentes le projet fini à l'enseignant
                         
🔄 WORKFLOW GIT POUR CHAQUE MEMBRE
Commandes à exécuter CHAQUE JOUR :
bash
# 1. Récupérer les dernières modifications
git pull origin main

# 2. Aller sur SA branche
git checkout VOTRE_BRANCHE

# 3. Fusionner main dans votre branche
git merge main

# 4. Travailler sur vos fichiers...

# 5. Tester
cargo test
cargo clippy -- -D warnings

# 6. Commiter
git add .
git commit -m "feat: description de ce que vous avez fait"

# 7. Pousser
git push origin VOTRE_BRANCHE
📅 PLANNING DÉTAILLÉ
SAMEDI (Jour 1)
08h00-09h00 : TSEFACK initialise le projet, tout le monde clone

09h00-13h00 : Chacun travaille sur SES fichiers

13h00-14h00 : Pause + synchronisation Git

14h00-18h00 : Suite du travail individuel

18h00-20h00 : Tests unitaires, résolution de bugs

20h00-22h00 : Première tentative d'intégration

DIMANCHE (Jour 2)
08h00-12h00 : Finalisation, tests, corrections

12h00-14h00 : Tests d'intégration

14h00-18h00 : Bonus, documentation, exemples

18h00-20h00 : Tests finaux

20h00-22h00 : Dernières corrections

LUNDI MATIN
06h00-08h00 : Intégration finale

08h00-10h00 : Tests complets, résolution conflits

10h00-12h00 : Préparation démo, rapport HTML, soumission

✅ CHECKLIST PAR MEMBRE
NZEUTEM (#02)
Lecteur CSV fonctionnel

Tests unitaires qui passent

Gère les délimiteurs personnalisés

Fichier test CSV créé

DIOM (#03)
Lecteur JSON fonctionnel

Lecteur texte délimité fonctionnel

Tests unitaires qui passent

Fichiers test créés

ASSONGUE (#04)
Trait Transform défini

Rename fonctionnel

Filter fonctionnel avec tous les opérateurs

Tests unitaires qui passent

NOLACK (#05)
Cast fonctionnel (string/number/boolean)

Compute fonctionnel

Drop fonctionnel

Factory fonctionnelle

Tests unitaires qui passent

NGLITANG (#06)
Trait SinkWriter défini

CSV writer fonctionnel avec BufWriter

JSON writer fonctionnel

Tests unitaires qui passent

NGANSOP (#07)
JSONL writer fonctionnel (streaming)

Factory d'écrivains fonctionnelle

Tests d'intégration complets

Test avec 1000+ lignes

DONFACK (#08)
Stats avec tous les compteurs

Rapport terminal avec couleurs

Validation de schéma

Rapport HTML généré

Tests unitaires qui passent

ATEKOUMBO (#09)
Jointure INNER fonctionnelle

Jointure LEFT fonctionnelle

Mode dry-run fonctionnel

Exemple join_demo créé

Tests unitaires qui passent

NJOH (#10)
Mode watch fonctionnel

Exemples créés (basic + hospital)

Données de test réalistes

Documentation cargo doc propre

TSEFACK (#01)
Structure du projet complète

Cargo.toml avec toutes les dépendances

CLI clap dans main.rs

Config TOML parsée correctement

Orchestrateur pipeline.rs fonctionnel

README.md complet

Intégration de toutes les parties

Tests finaux qui passent

🚨 RÈGLES IMPORTANTES
Commitez TOUTES les 30 minutes - c'est votre sauvegarde

Faites cargo test avant chaque commit

Pas de unwrap() dans le code final - utilisez ? ou match

Documentez avec /// - ça génère la doc automatique

Utilisez cargo fmt avant de commiter

Si bloqué, demandez dans le groupe WhatsApp

NE MODIFIEZ PAS les fichiers des autres sans coordination

📞 CONTACTS
Chef de projet : TSEFACK CALVIN KLEIN

Groupe WhatsApp : [Lien du groupe]

GitHub : [Lien du repo]

BONNE CHANCE À TOUTE L'ÉQUIPE ! 🚀

text

Pour ajouter ce fichier au projet :

```bash
# Créer le fichier
cat > GUIDE_TRAVAIL_EQUIPE.md << 'EOF'
[Coller tout le contenu ci-dessus]
EOF

# Ajouter et commiter
git add GUIDE_TRAVAIL_EQUIPE.md
git commit -m "docs: Guide de travail détaillé pour chaque membre de l'équipe"
git push origin main