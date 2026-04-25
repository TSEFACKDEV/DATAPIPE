use std::time::Instant;

#[derive(Default, Debug)]
pub struct ExecutionStats {
    pub records_read: u64,
    pub records_transformed: u64,
    pub records_filtered: u64,
    pub records_written: u64,
    pub errors_encountered: u64,
    pub start_time: Option<Instant>,
    pub duration_ms: u64,
}

impl ExecutionStats {
    pub fn new() -> Self {
        ExecutionStats {
            start_time: Some(Instant::now()),
            ..Default::default()
        }
    }

    pub fn print_report(&self) {
        // TODO: Implémenter l'affichage (DONFACK #08)
        println!("=== RAPPORT D'EXÉCUTION DATAPIPE ===");
        println!("Records lus : {}", self.records_read);
        println!("Records transformés: {}", self.records_transformed);
        println!("Records filtrés : {}", self.records_filtered);
        println!("Records écrits : {}", self.records_written);
        println!("Erreurs : {}", self.errors_encountered);
        println!("Durée : {}ms", self.duration_ms);
    }

    pub fn stop(&mut self) {
        if let Some(start) = self.start_time {
            self.duration_ms = start.elapsed().as_millis() as u64;
        }
    }
}