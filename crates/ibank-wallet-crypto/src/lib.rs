//! Signing interfaces and wallet-core bridge.

use ibank_wallet_chains::EvmUnsignedTx;
use ibank_wallet_core::{Result, WalletError};

#[cfg(feature = "wallet-core")]
mod wallet_core;

/// A signer capable of producing signed EVM transactions.
pub trait Signer {
    /// Signs an EIP-1559 transaction and returns the signed bytes.
    fn sign_evm_eip1559(&self, chain_id: &str, tx: &EvmUnsignedTx) -> Result<Vec<u8>>;
}

#[cfg(feature = "wallet-core")]
pub use wallet_core::WalletCoreSigner;

/// Mock signer used when wallet-core is disabled.
#[cfg(not(feature = "wallet-core"))]
#[derive(Clone, Debug, Default)]
pub struct MockSigner;

#[cfg(not(feature = "wallet-core"))]
impl Signer for MockSigner {
    fn sign_evm_eip1559(&self, _chain_id: &str, tx: &EvmUnsignedTx) -> Result<Vec<u8>> {
        let mut payload = tx.signing_payload();
        if payload.is_empty() {
            return Err(WalletError::SigningError("empty payload".to_string()));
        }
        payload.extend_from_slice(b"mock");
        Ok(payload)
    }
}
