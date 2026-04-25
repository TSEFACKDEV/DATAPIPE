// src/writer/jsonl_writer.rs
use super::{Record, SinkWriter};
use std::io::Write;

pub struct JsonLinesSinkWriter<W: Write> {
    writer: W,
}

impl<W: Write> SinkWriter for JsonLinesSinkWriter<W> {
    fn write_record(&mut self, record: &Record) -> anyhow::Result<()> {
        // TODO: Implémenter l'écriture JSONL (NGANSOP #07)
        // 1. Sérialiser le record en JSON
        // 2. Écrire une ligne
        todo!("Implémenter l'écrivain JSONL")
    }

    fn finalize(&mut self) -> anyhow::Result<()> {
        // TODO: Flush final
        todo!("Implémenter finalize JSONL")
    }
}