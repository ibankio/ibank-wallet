//! Error types for ibank-wallet.

use thiserror::Error;

/// A specialized result type for wallet operations.
pub type Result<T> = std::result::Result<T, WalletError>;

/// Error variants returned by wallet APIs.
#[derive(Debug, Error)]
pub enum WalletError {
    /// Indicates a policy denial.
    #[error("policy violation: {0}")]
    PolicyViolation(String),
    /// Indicates signing failure.
    #[error("signing error: {0}")]
    SigningError(String),
    /// Indicates RPC or submission failure.
    #[error("rpc error: {0}")]
    RpcError(String),
    /// Indicates invalid input.
    #[error("invalid input: {0}")]
    InvalidInput(String),
}
