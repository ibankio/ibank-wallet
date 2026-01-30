#[cfg(feature = "wallet-core")]
mod wallet_core_tests {
    use ibank_wallet_chains::EvmUnsignedTx;
    use ibank_wallet_crypto::{Signer, WalletCoreSigner};

    const MNEMONIC: &str =
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    const DEFAULT_PATH: &str = "m/44'/60'/0'/0/0";

    #[test]
    fn derives_expected_evm_address() {
        let signer = WalletCoreSigner::from_mnemonic(MNEMONIC, "").expect("signer");
        let address = signer.evm_address(Some(DEFAULT_PATH)).expect("address");
        let expected = hex_to_bytes("27ef5cdbe01777d62438affeb695e33fc2335979");
        assert_eq!(address.as_slice(), expected.as_slice());
    }

    #[test]
    fn signs_eip1559_transaction() {
        let signer = WalletCoreSigner::from_mnemonic(MNEMONIC, "").expect("signer");
        let tx = EvmUnsignedTx {
            chain_id: 1,
            nonce: 0,
            max_priority_fee_per_gas: 1,
            max_fee_per_gas: 2,
            gas_limit: 21_000,
            to: Some([0x11u8; 20]),
            value: 1,
            data: Vec::new(),
            access_list: Default::default(),
        };
        let signed = signer.sign_evm_eip1559("eip155:1", &tx).expect("signed");
        assert!(!signed.is_empty());
        assert_eq!(signed.first().copied(), Some(0x02));
    }

    fn hex_to_bytes(hex: &str) -> Vec<u8> {
        let mut out = Vec::with_capacity(hex.len() / 2);
        let mut chars = hex.chars();
        while let (Some(hi), Some(lo)) = (chars.next(), chars.next()) {
            let byte = (hex_value(hi) << 4) | hex_value(lo);
            out.push(byte);
        }
        out
    }

    fn hex_value(c: char) -> u8 {
        match c {
            '0'..='9' => c as u8 - b'0',
            'a'..='f' => c as u8 - b'a' + 10,
            'A'..='F' => c as u8 - b'A' + 10,
            _ => 0,
        }
    }
}
