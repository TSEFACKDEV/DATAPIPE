// src/lib.rs
//
// Point d'entrée de la BIBLIOTHÈQUE (library crate) DataPipe.
//
// ─── POURQUOI LIB.RS ET MAIN.RS COEXISTENT-ILS ? ────────────────────────────
//
// En Rust, un projet peut être SIMULTANÉMENT :
//   - Un binaire (main.rs → compile vers un exécutable `datapipe`)
//   - Une bibliothèque (lib.rs → expose des modules testables)
//
// Pourquoi cette dualité ?
//   Les tests d'intégration dans tests/ traitent le code comme une BIBLIOTHÈQUE
//   externe : ils importent via `use datapipe::...`. Sans lib.rs, les modules
//   de src/ ne sont pas accessibles depuis tests/.
//
// Principe : main.rs est le point d'entrée CLI, lib.rs est l'API testable.
// main.rs appelle les fonctions de lib.rs, et les tests font de même.
// ────────────────────────────────────────────────────────────────────────────

pub mod config;
pub mod reader;
pub mod writer;
pub mod transform;
pub mod stats;
pub mod validation;
pub mod report;
pub mod join;
pub mod pipeline;
pub mod watch;
