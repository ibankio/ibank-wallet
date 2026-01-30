//! wallet-core FFI bridge.

#[cxx::bridge]
mod ffi {
    unsafe extern "C++" {
        include!("ffi.h");

        type WalletCoreSigner;

        fn new_signer(mnemonic: &str, passphrase: &str) -> UniquePtr<WalletCoreSigner>;

        fn derive_evm_address(signer: &WalletCoreSigner, derivation_path: &str) -> Vec<u8>;

        fn sign_eip1559(
            signer: &WalletCoreSigner,
            chain_id: u64,
            nonce: u64,
            max_priority_fee_per_gas_be: &Vec<u8>,
            max_fee_per_gas_be: &Vec<u8>,
            gas_limit_be: &Vec<u8>,
            to20: &Vec<u8>,
            value_be: &Vec<u8>,
            data: &Vec<u8>,
            access_list_rlp: &Vec<u8>,
        ) -> Vec<u8>;
    }
}

pub use ffi::{derive_evm_address, new_signer, sign_eip1559, WalletCoreSigner};
