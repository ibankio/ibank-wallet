//! wallet-core FFI bridge.

#[cxx::bridge]
mod ffi {
    extern "C++" {
        include!("ffi.h");

        type WalletCoreSigner;

        fn new_signer(mnemonic: &str, passphrase: &str) -> UniquePtr<WalletCoreSigner>;
        fn derive_evm_address(signer: &WalletCoreSigner, derivation_path: &str) -> Vec<u8>;
        fn sign_eip1559(
            signer: &WalletCoreSigner,
            chain_id: u64,
            nonce: u64,
            max_priority_fee_per_gas: &Vec<u8>,
            max_fee_per_gas: &Vec<u8>,
            gas_limit: &Vec<u8>,
            to: &Vec<u8>,
            value: &Vec<u8>,
            data: &Vec<u8>,
            access_list: &Vec<u8>,
        ) -> Vec<u8>;
    }
}

pub use ffi::{derive_evm_address, new_signer, sign_eip1559, WalletCoreSigner};
