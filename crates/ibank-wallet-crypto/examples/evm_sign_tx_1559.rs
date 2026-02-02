use ibank_wallet_crypto::wallet_core::{WalletCoreSigner, Evm1559Tx};

fn to_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        s.push(HEX[(b >> 4) as usize] as char);
        s.push(HEX[(b & 0x0f) as usize] as char);
    }
    s
}

fn main() {
    let mnemonic = "test test test test test test test test test test test junk";
    let passphrase = "";
    let path = "m/44'/60'/0'/0/0";

    let signer = WalletCoreSigner::from_mnemonic(mnemonic, passphrase).unwrap();
    let from = signer.derive_evm_address(path).unwrap();
    println!("from: 0x{}", to_hex(&from));

    // A minimal EIP-1559 transfer
    let tx = Evm1559Tx {
        chain_id: 1,
        nonce: 0,
        max_priority_fee_per_gas: 1_500_000_000u64, // 1.5 gwei
        max_fee_per_gas: 30_000_000_000u64,         // 30 gwei
        gas_limit: 21_000u64,
        to: hex::decode("d8da6bf26964af9d7eed9e03e53415d37aa96045").unwrap(), // 20 bytes
        value: 1_000_000_000_000_000u64, // 0.001 ETH
        data: vec![],
        access_list_rlp: vec![], // can be empty for now
    };

    // You implement this method: signer.sign_evm_1559_tx(path, &tx) -> Vec<u8> (raw tx)
    let raw = signer.sign_evm_1559_tx(path, &tx).unwrap();

    println!("raw signed tx (hex): 0x{}", to_hex(&raw));
}