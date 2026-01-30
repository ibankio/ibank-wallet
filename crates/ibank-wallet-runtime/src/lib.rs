//! Intent-to-submit runtime orchestrator.

use ibank_wallet_chains::{AccessList, EvmUnsignedTx};
use ibank_wallet_core::{AuditEvent, AuditLog, CaipChainId, Result, WalletError};
use ibank_wallet_crypto::Signer;
use ibank_wallet_policy::{enforce, PolicyEngine};
use serde::{Deserialize, Serialize};
use serde_json::json;

/// Transfer intent for EVM chains.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Intent {
    /// CAIP-2 chain id (e.g. eip155:1).
    pub chain_id: CaipChainId,
    /// Sender nonce.
    pub nonce: u64,
    /// Recipient address.
    pub to: [u8; 20],
    /// Value in wei.
    pub value: u128,
    /// Calldata payload.
    pub data: Vec<u8>,
}

/// Quote placeholder for gas estimates.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Quote {
    /// Max priority fee per gas.
    pub max_priority_fee_per_gas: u128,
    /// Max fee per gas.
    pub max_fee_per_gas: u128,
    /// Gas limit.
    pub gas_limit: u128,
    /// Access list (optional).
    pub access_list: AccessList,
}

/// Runtime orchestrator for intents.
#[derive(Debug)]
pub struct Runtime<P, S> {
    /// Policy engine.
    pub policy: P,
    /// Signer implementation.
    pub signer: S,
    /// Audit log.
    pub audit_log: AuditLog,
}

impl<P, S> Runtime<P, S>
where
    P: PolicyEngine,
    S: Signer,
{
    /// Creates a new runtime instance.
    pub fn new(policy: P, signer: S) -> Self {
        Self {
            policy,
            signer,
            audit_log: AuditLog::default(),
        }
    }

    /// Signs an intent after policy evaluation and audit logging.
    pub fn sign_intent(&mut self, intent: &Intent, quote: &Quote) -> Result<Vec<u8>> {
        let chain_id = parse_chain_id(&intent.chain_id)?;
        let tx = EvmUnsignedTx {
            chain_id,
            nonce: intent.nonce,
            max_priority_fee_per_gas: quote.max_priority_fee_per_gas,
            max_fee_per_gas: quote.max_fee_per_gas,
            gas_limit: quote.gas_limit,
            to: Some(intent.to),
            value: intent.value,
            data: intent.data.clone(),
            access_list: quote.access_list.clone(),
        };

        let decision = self.policy.evaluate_evm(&tx)?;
        enforce(decision)?;

        let signed = self
            .signer
            .sign_evm_eip1559(intent.chain_id.as_str(), &tx)?;

        self.audit_log.record(AuditEvent {
            name: "sign_evm_eip1559".to_string(),
            metadata: json!({
                "chain_id": intent.chain_id.as_str(),
                "nonce": intent.nonce,
                "to": hex::encode(intent.to),
                "value": intent.value,
            }),
        });

        Ok(signed)
    }
}

fn parse_chain_id(chain_id: &CaipChainId) -> Result<u64> {
    let value = chain_id.as_str();
    let trimmed = value
        .strip_prefix("eip155:")
        .ok_or_else(|| WalletError::InvalidInput("unsupported chain id".to_string()))?;
    trimmed
        .parse::<u64>()
        .map_err(|_| WalletError::InvalidInput("invalid chain id".to_string()))
}
