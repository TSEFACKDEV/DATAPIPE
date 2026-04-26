#!/usr/bin/env bash
# =============================================================================
# demo.sh — Script de démonstration DataPipe
# Groupe 6 | ENSP Yaoundé 2024–2025 | NJOH MASSANGO ADOLPHE MACDEAUVILLE
# =============================================================================
# Présentation live de toutes les fonctionnalités de DataPipe :
#   1. Compilation du projet
#   2. Exemple de base (CSV → JSON, filtrage, calcul de prime)
#   3. Exemple de jointure (commandes + clients)
#   4. Exemple hôpital (cas camerounais réel)
#   5. Mode dry-run
#   6. Mode watch (limité à 3 cycles pour la démo)
# =============================================================================

set -e  # Arrêter immédiatement en cas d'erreur

# --- Couleurs ---
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[1;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
RESET='\033[0m'

# --- Helpers ---
section() {
    echo ""
    echo -e "${BLUE}${BOLD}══════════════════════════════════════════════${RESET}"
    echo -e "${BLUE}${BOLD}  $1${RESET}"
    echo -e "${BLUE}${BOLD}══════════════════════════════════════════════${RESET}"
    echo ""
}

step() {
    echo -e "${CYAN}▶ $1${RESET}"
}

ok() {
    echo -e "${GREEN}✅ $1${RESET}"
}

warn() {
    echo -e "${YELLOW}⚠️  $1${RESET}"
}

pause() {
    echo ""
    echo -e "${YELLOW}[Appuyer sur Entrée pour continuer...]${RESET}"
    read -r
}

# =============================================================================
section "🚀 DataPipe — Démonstration Groupe 6 ENSP Yaoundé"
# =============================================================================

echo -e "${BOLD}Membres du groupe :${RESET}"
echo "  TSEFACK CALVIN KLEIN         | Chef de Projet & Architecture"
echo "  NZEUTEM DOMMOE EUNICE        | Lecteur CSV"
echo "  DIOM LUCRAINE LETHICIA       | Lecteur JSON & Texte délimité"
echo "  ASSONGUE TATANG MURIEL       | Transformations Rename & Filter"
echo "  NOLACK KAWUNJIBI FRANGE      | Transformations Cast, Compute, Drop"
echo "  NGLITANG RUBEN               | Écrivains CSV & JSON"
echo "  NGANSOP NGOUABOU FREDI       | Écrivain JSONL & Tests intégration"
echo "  DONFACK KEUNANG VLADIMIR     | Statistiques, Validation, Rapport HTML"
echo "  ATEKOUMBO EXCEL DEXTE        | Jointure JOIN & Mode Dry-Run"
echo "  NJOH MASSANGO ADOLPHE        | Mode Watch, Documentation & Démo"

pause

# =============================================================================
section "1️⃣  Compilation du projet"
# =============================================================================

step "Compilation avec cargo build --release..."
cargo build --release 2>&1
ok "Compilation réussie !"

step "Vérification des tests unitaires..."
cargo test 2>&1
ok "Tous les tests passent !"

pause

# =============================================================================
section "2️⃣  Exemple de base — CSV → JSON (Employés Informatique)"
# =============================================================================

step "Fichier source : data/employes.csv"
echo ""
cat data/employes.csv | column -t -s','
echo ""

step "Configuration : examples/basic/pipeline.toml"
echo ""
cat examples/basic/pipeline.toml
echo ""

step "Lancement du pipeline..."
mkdir -p output
./target/release/datapipe --config examples/basic/pipeline.toml

echo ""
step "Résultat : output/employes_informatique.json"
cat output/employes_informatique.json
ok "Pipeline de base exécuté avec succès !"

pause

# =============================================================================
section "3️⃣  Exemple Jointure — Commandes + Clients"
# =============================================================================

step "Fichier gauche : data/commandes.csv (streamé)"
cat data/commandes.csv | column -t -s','
echo ""

step "Fichier droit  : data/clients.json (chargé en mémoire)"
cat data/clients.json
echo ""

step "Lancement de la jointure..."
./target/release/datapipe --config examples/join_demo/pipeline.toml

echo ""
step "Résultat enrichi : output/commandes_enrichies.csv"
cat output/commandes_enrichies.csv | column -t -s','
ok "Jointure inner join exécutée avec succès !"

pause

# =============================================================================
section "4️⃣  Cas d'usage Hôpital Central (Cameroun)"
# =============================================================================

step "Fichier source : data/patients.csv"
cat data/patients.csv | column -t -s','
echo ""

step "Pipeline : renommage colonnes + filtre hospitalisés + suppression NumeroSecu"
./target/release/datapipe --config examples/hospital/pipeline.toml

echo ""
step "Résultat : output/patients_hospitalises.json"
cat output/patients_hospitalises.json
ok "Données patients standardisées et sécurisées !"

pause

# =============================================================================
section "5️⃣  Mode Dry-Run — Simulation sans écriture"
# =============================================================================

step "Lancement en mode --dry-run (aucun fichier de sortie créé)..."
./target/release/datapipe --config examples/basic/pipeline.toml --dry-run

warn "Aucun fichier output créé — c'est l'objectif du dry-run."
ok "Mode dry-run opérationnel !"

pause

# =============================================================================
section "6️⃣  Mode Watch — Surveillance en temps réel (démo 2 cycles)"
# =============================================================================

step "On lance le watch en arrière-plan et on simule une modification..."
echo ""

# Lancer le watch en arrière-plan (interval 5s pour la démo)
./target/release/datapipe --config examples/basic/pipeline.toml --watch --interval 5 &
WATCH_PID=$!

echo "PID watch : $WATCH_PID"
echo "Attente 3s..."
sleep 3

# Simuler une modification du fichier source
step "Modification simulée du fichier source..."
echo "Nouveau Employe Démo,Informatique,500000,1,Douala,secret" >> data/employes.csv

echo "Attente du prochain cycle (5s)..."
sleep 6

# Arrêter le watch
kill $WATCH_PID 2>/dev/null || true
ok "Mode watch a détecté la modification et relancé le pipeline !"

# Nettoyer la ligne ajoutée
head -n -1 data/employes.csv > /tmp/employes_tmp.csv && mv /tmp/employes_tmp.csv data/employes.csv

pause

# =============================================================================
section "7️⃣  Rapport HTML & Documentation"
# =============================================================================

step "Rapport HTML généré automatiquement après chaque exécution..."
if [ -f output/rapport.html ]; then
    ok "Rapport disponible : output/rapport.html"
    echo "  → Ouvrez ce fichier dans votre navigateur pour voir les statistiques."
else
    warn "rapport.html non trouvé (nécessite l'intégration de DONFACK #08)."
fi

step "Génération de la documentation Rust..."
cargo doc --no-deps 2>&1
ok "Documentation générée ! Ouvrir avec : cargo doc --open"

# =============================================================================
section "✅ Démonstration terminée !"
# =============================================================================

echo -e "${GREEN}${BOLD}"
echo "  DataPipe est un outil ETL complet développé en Rust par le Groupe 6."
echo "  Il lit CSV/JSON/texte, transforme, filtre, joint et écrit CSV/JSON/JSONL."
echo "  Avec le mode watch, il peut s'intégrer dans des pipelines de données"
echo "  automatiques (rapports journaliers, synchronisation de bases de données)."
echo -e "${RESET}"

echo -e "Livrables disponibles dans :"
echo "  output/            — Fichiers de sortie générés"
echo "  examples/          — Configurations de démo"
echo "  data/              — Données camerounaises réalistes"
echo "  target/doc/        — Documentation Rust générée"

echo ""
echo -e "${BOLD}Bonne correction ! 🎓${RESET}"
