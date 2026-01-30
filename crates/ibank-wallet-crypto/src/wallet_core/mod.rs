//! wallet-core backed signer implementation.

use ibank_wallet_chains::EvmUnsignedTx;
use ibank_wallet_core::{Result, WalletError};

use crate::Signer;

mod ffi;

/// Signer implementation backed by Trust Wallet wallet-core.
// #[derive(Debug)]
pub struct WalletCoreSigner {
    inner: cxx::UniquePtr<ffi::WalletCoreSigner>,
}

impl WalletCoreSigner {
    /// Creates a signer from a mnemonic and optional passphrase.
    pub fn from_mnemonic(mnemonic: &str, passphrase: &str) -> Result<Self> {
        let inner = ffi::new_signer(mnemonic, passphrase);
        if inner.is_null() {
            return Err(WalletError::SigningError(
                "failed to create wallet-core signer".to_string(),
            ));
        }
        Ok(Self { inner })
    }

    /// Derives the EVM address at the given derivation path.
    pub fn derive_evm_address(&self, derivation_path: &str) -> Result<[u8; 20]> {
        let address = ffi::derive_evm_address(&self.inner, derivation_path);
        if address.len() != 20 {
            return Err(WalletError::SigningError(
                "wallet-core returned invalid address".to_string(),
            ));
        }
        let mut out = [0u8; 20];
        out.copy_from_slice(&address);
        Ok(out)
    }
}

impl Signer for WalletCoreSigner {
    fn sign_evm_eip1559(&self, chain_id: &str, tx: &EvmUnsignedTx) -> Result<Vec<u8>> {
        let chain_id = parse_chain_id(chain_id)?;
        let to_bytes = tx.to.map(|addr| addr.to_vec()).unwrap_or_default();
        let max_priority_fee_per_gas = u128_to_bytes(tx.max_priority_fee_per_gas);
        let max_fee_per_gas = u128_to_bytes(tx.max_fee_per_gas);
        let gas_limit = u128_to_bytes(tx.gas_limit);
        let value = u128_to_bytes(tx.value);
        let access_list = tx.access_list.0.iter().flat_map(|item| {
            let mut entry = Vec::new();
            entry.extend_from_slice(&item.address);
            for key in &item.storage_keys {
                entry.extend_from_slice(key);
            }
            entry
        }).collect::<Vec<u8>>();

        let signed = ffi::sign_eip1559(
            &self.inner,
            chain_id,
            tx.nonce,
            &max_priority_fee_per_gas,
            &max_fee_per_gas,
            &gas_limit,
            &to_bytes,
            &value,
            &tx.data,
            &access_list,
        );

        if signed.is_empty() {
            return Err(WalletError::SigningError(
                "wallet-core returned empty signature".to_string(),
            ));
        }
        Ok(signed)
    }
}

fn parse_chain_id(chain_id: &str) -> Result<u64> {
    let trimmed = chain_id.strip_prefix("eip155:").unwrap_or(chain_id);
    trimmed
        .parse::<u64>()
        .map_err(|_| WalletError::InvalidInput("invalid chain id".to_string()))
}

fn u128_to_bytes(value: u128) -> Vec<u8> {
    if value == 0 {
        return Vec::new();
    }
    let bytes = value.to_be_bytes();
    let first_nonzero = bytes.iter().position(|b| *b != 0).unwrap_or(bytes.len());
    bytes[first_nonzero..].to_vec()
}
