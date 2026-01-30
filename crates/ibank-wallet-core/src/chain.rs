//! Chain identifiers.

use serde::{Deserialize, Serialize};

/// CAIP-2 chain identifier wrapper (e.g. "eip155:1").
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct CaipChainId(pub String);

impl CaipChainId {
    /// Creates a new CAIP-2 chain identifier.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the underlying string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
