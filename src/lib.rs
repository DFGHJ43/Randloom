//! Randloom — interactive terminal random number generator.
//!
//! The crate is split into a terminal-free core (`rng`, `dist`, `config`,
//! `stats`, `fmt`, `export`), a headless render model (`ui`), an application
//! state machine (`app`) and a thin crossterm-based terminal layer (`term`,
//! driven by `run`).

#![forbid(unsafe_code)]

pub mod app;
pub mod config;
pub mod dist;
pub mod export;
pub mod fmt;
pub mod input;
pub mod rng;
pub mod stats;
pub mod ui;

mod run;
mod term;

pub use run::run;
