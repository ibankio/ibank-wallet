//! Policy engine skeleton.

use ibank_wallet_chains::EvmUnsignedTx;
use ibank_wallet_core::{Result, WalletError};
use serde::{Deserialize, Serialize};

/// Policy decision result for an intent.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PolicyDecision {
    /// True if the action is allowed.
    pub allowed: bool,
    /// Optional reason for denial.
    pub reason: Option<String>,
}

/// A policy engine that can evaluate EVM transactions.
pub trait PolicyEngine {
    /// Evaluates an EVM transaction intent.
    fn evaluate_evm(&self, tx: &EvmUnsignedTx) -> Result<PolicyDecision>;
}

/// Minimal spend limit policy stub.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpendLimitPolicy {
    /// Maximum allowed value in wei.
    pub max_value: u128,
}

impl PolicyEngine for SpendLimitPolicy {
    fn evaluate_evm(&self, tx: &EvmUnsignedTx) -> Result<PolicyDecision> {
        if tx.value > self.max_value {
            return Ok(PolicyDecision {
                allowed: false,
                reason: Some("value exceeds spend limit".to_string()),
            });
        }
        Ok(PolicyDecision {
            allowed: true,
            reason: None,
        })
    }
}

/// Basic allowlist stub that currently permits all recipients.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AllowListPolicy;

impl PolicyEngine for AllowListPolicy {
    fn evaluate_evm(&self, _tx: &EvmUnsignedTx) -> Result<PolicyDecision> {
        Ok(PolicyDecision {
            allowed: true,
            reason: None,
        })
    }
}

/// Enforces policy decision or returns an error.
pub fn enforce(decision: PolicyDecision) -> Result<()> {
    if decision.allowed {
        Ok(())
    } else {
        Err(WalletError::PolicyViolation(
            decision.reason.unwrap_or_else(|| "policy denied".to_string()),
        ))
    }
}
