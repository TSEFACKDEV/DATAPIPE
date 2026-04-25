use crate::reader::Record;
use std::collections::HashMap;

pub fn build_lookup(
    records: impl Iterator<Item = anyhow::Result<Record>>,
    key: &str,
) -> HashMap<String, Record> {
    // TODO: Implémenter la construction du lookup (ATEKOUMBO #09)
    // 1. Parcourir les records
    // 2. Créer un HashMap avec la clé spécifiée
    todo!("Implémenter build_lookup")
}

pub fn join_records(
    left: Record,
    right_lookup: &HashMap<String, Record>,
    left_key: &str,
    join_type: &str,
) -> Option<Record> {
    // TODO: Implémenter la jointure (ATEKOUMBO #09)
    // 1. Récupérer la clé de gauche
    // 2. Chercher dans le lookup
    // 3. Fusionner les records si trouvé
    todo!("Implémenter join_records")
}