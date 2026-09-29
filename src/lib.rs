//! Cross-platform process and firewall orchestration for SuZapret.

pub mod error;
pub mod strategy;
pub mod backend;

pub use backend::{Backend, ZapretBackend};
pub use error::{Result, SuZapretError};
pub use strategy::{Strategy, StrategyStore};
