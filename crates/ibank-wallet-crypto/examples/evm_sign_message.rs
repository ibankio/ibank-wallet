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
    let mnemonic = "test test test test test test test test test test test junk";
    let passphrase = "";
    let path = "m/44'/60'/0'/0/0";

    let signer = WalletCoreSigner::from_mnemonic(mnemonic, passphrase)
        .expect("failed to create signer");

    let addr20 = signer.derive_evm_address(path).expect("addr");
    println!("address: 0x{}", to_hex(&addr20));

    // Example: "hello"
    let msg = b"hello";

    // You need to implement this method (see below): signer.sign_evm_personal_message(path, msg)
    let sig65 = signer
        .sign_evm_personal_message(path, msg)
        .expect("sign message");

    println!("signature (65 bytes): 0x{}", to_hex(&sig65));
}