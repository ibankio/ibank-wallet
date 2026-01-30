//! EVM chain types and signing payload builder.

use serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};

/// An EVM access list item.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct AccessListItem {
    /// The accessed address.
    pub address: [u8; 20],
    /// The storage keys accessed for the address.
    pub storage_keys: Vec<[u8; 32]>,
}

/// EIP-2930/EIP-1559 access list.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct AccessList(pub Vec<AccessListItem>);

/// Unsigned EVM transaction for EIP-1559 signing.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct EvmUnsignedTx {
    /// Chain id for replay protection.
    pub chain_id: u64,
    /// Sender nonce.
    pub nonce: u64,
    /// Max priority fee per gas (aka tip).
    pub max_priority_fee_per_gas: u128,
    /// Max fee per gas.
    pub max_fee_per_gas: u128,
    /// Gas limit.
    pub gas_limit: u128,
    /// Recipient address, or None for contract creation.
    pub to: Option<[u8; 20]>,
    /// Value transferred in wei.
    pub value: u128,
    /// Call data.
    pub data: Vec<u8>,
    /// Access list.
    pub access_list: AccessList,
}

impl EvmUnsignedTx {
    /// Builds the EIP-1559 signing payload bytes: 0x02 || rlp([...]).
    pub fn signing_payload(&self) -> Vec<u8> {
        let mut stream = rlp::RlpStream::new_list(9);
        stream.append(&self.chain_id);
        stream.append(&self.nonce);
        append_u128(&mut stream, self.max_priority_fee_per_gas);
        append_u128(&mut stream, self.max_fee_per_gas);
        append_u128(&mut stream, self.gas_limit);
        match self.to {
            Some(address) => stream.append(&address.as_slice()),
            None => stream.append(&Vec::<u8>::new()),
        };
        append_u128(&mut stream, self.value);
        stream.append(&self.data.as_slice());
        append_access_list(&mut stream, &self.access_list);

        let mut out = Vec::with_capacity(1 + stream.out().len());
        out.push(0x02);
        out.extend_from_slice(stream.out().as_ref());
        out
    }

    /// Hashes the signing payload with keccak256.
    pub fn signing_payload_hash(&self) -> [u8; 32] {
        keccak256(&self.signing_payload())
    }
}

/// Helper builder for EVM unsigned transactions.
#[derive(Clone, Debug, Default)]
pub struct EvmUnsignedTxBuilder {
    tx: EvmUnsignedTx,
}

impl EvmUnsignedTxBuilder {
    /// Creates a new builder with required fields.
    pub fn new(chain_id: u64, nonce: u64) -> Self {
        Self {
            tx: EvmUnsignedTx {
                chain_id,
                nonce,
                max_priority_fee_per_gas: 0,
                max_fee_per_gas: 0,
                gas_limit: 0,
                to: None,
                value: 0,
                data: Vec::new(),
                access_list: AccessList::default(),
            },
        }
    }

    /// Sets max priority fee per gas.
    pub fn max_priority_fee_per_gas(mut self, value: u128) -> Self {
        self.tx.max_priority_fee_per_gas = value;
        self
    }

    /// Sets max fee per gas.
    pub fn max_fee_per_gas(mut self, value: u128) -> Self {
        self.tx.max_fee_per_gas = value;
        self
    }

    /// Sets gas limit.
    pub fn gas_limit(mut self, value: u128) -> Self {
        self.tx.gas_limit = value;
        self
    }

    /// Sets recipient.
    pub fn to(mut self, value: [u8; 20]) -> Self {
        self.tx.to = Some(value);
        self
    }

    /// Sets value.
    pub fn value(mut self, value: u128) -> Self {
        self.tx.value = value;
        self
    }

    /// Sets data.
    pub fn data(mut self, value: Vec<u8>) -> Self {
        self.tx.data = value;
        self
    }

    /// Sets access list.
    pub fn access_list(mut self, value: AccessList) -> Self {
        self.tx.access_list = value;
        self
    }

    /// Returns the built transaction.
    pub fn build(self) -> EvmUnsignedTx {
        self.tx
    }
}

/// Computes keccak256 digest of input.
pub fn keccak256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Keccak256::new();
    hasher.update(data);
    let result = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&result);
    out
}

fn append_u128(stream: &mut rlp::RlpStream, value: u128) {
    let bytes = u128_to_bytes(value);
    stream.append(&bytes);
}

fn u128_to_bytes(value: u128) -> Vec<u8> {
    if value == 0 {
        return Vec::new();
    }
    let bytes = value.to_be_bytes();
    let first_nonzero = bytes.iter().position(|b| *b != 0).unwrap_or(bytes.len());
    bytes[first_nonzero..].to_vec()
}

fn append_access_list(stream: &mut rlp::RlpStream, access_list: &AccessList) {
    stream.begin_list(access_list.0.len());
    for item in &access_list.0 {
        stream.begin_list(2);
        stream.append(&item.address.as_slice());
        stream.begin_list(item.storage_keys.len());
        for key in &item.storage_keys {
            stream.append(&key.as_slice());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eip1559_signing_payload_matches_expected() {
        let tx = EvmUnsignedTx {
            chain_id: 1,
            nonce: 0,
            max_priority_fee_per_gas: 1,
            max_fee_per_gas: 2,
            gas_limit: 21_000,
            to: Some([0u8; 20]),
            value: 1,
            data: Vec::new(),
            access_list: AccessList::default(),
        };

        let payload = tx.signing_payload();
        let expected = hex::decode(
            "02df018001028252089400000000000000000000000000000000000000000180c0",
        )
        .expect("valid hex");
        assert_eq!(payload, expected);
    }
}
