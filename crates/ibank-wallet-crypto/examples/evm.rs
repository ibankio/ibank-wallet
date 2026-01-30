use ibank_wallet_crypto::wallet_core::WalletCoreSigner;

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
    // Standard test mnemonic used across many wallets
    let mnemonic = "test test test test test test test test test test test junk";
    let passphrase = "";

    // Default EVM path
    let path = "m/44'/60'/0'/0/0";

    let signer = WalletCoreSigner::from_mnemonic(mnemonic, passphrase)
        .expect("failed to create signer");

    let addr20 = signer
        .derive_evm_address(path)
        .expect("failed to derive address");

    println!("path: {}", path);
    println!("address (20 bytes): 0x{}", to_hex(&addr20));
}