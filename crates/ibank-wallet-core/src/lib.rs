//! Core types and errors for ibank-wallet.

pub mod audit;
pub mod chain;
pub mod error;

pub use audit::{AuditEvent, AuditLog};
pub use chain::CaipChainId;
pub use error::{Result, WalletError};
