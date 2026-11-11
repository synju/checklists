//! checklists-app core: the shared JSON checklist model, filesystem store, and per-app config.
//!
//! This crate is the backend for the Electron + Vue desktop GUI, and is deliberately the same
//! JSON contract the original terminal app and Claude/agents use, so every tool reads and writes
//! one shared `checklists/` directory. Nothing here hardcodes that path — it comes from config.

pub mod config;
pub mod model;
pub mod service;
pub mod store;

pub use config::Config;
pub use model::{Checklist, Item, Progress};
pub use store::Store;
