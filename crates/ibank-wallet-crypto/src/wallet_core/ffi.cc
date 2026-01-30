#include "ffi.h"

#include <string>
#include <vector>

#include <TrustWalletCore/TWAnySigner.h>
#include <TrustWalletCore/TWCoinType.h>
#include <TrustWalletCore/TWData.h>
#include <TrustWalletCore/TWDerivationPath.h>
#include <TrustWalletCore/TWEthereum.h>
#include <TrustWalletCore/TWEthereumProto.h>
#include <TrustWalletCore/TWHDWallet.h>
#include <TrustWalletCore/TWPrivateKey.h>
#include <TrustWalletCore/TWPublicKey.h>
#include <TrustWalletCore/TWString.h>

namespace {
constexpr const char* kDefaultEvmDerivationPath = "m/44'/60'/0'/0/0";

struct SignerState {
  TWHDWallet* wallet = nullptr;
};

std::vector<uint8_t> to_big_endian(uint64_t value) {
  if (value == 0) {
    return {};
  }
  std::vector<uint8_t> out;
  out.reserve(sizeof(uint64_t));
  for (int i = 7; i >= 0; --i) {
    uint8_t byte = static_cast<uint8_t>((value >> (i * 8)) & 0xff);
    if (out.empty() && byte == 0) {
      continue;
    }
    out.push_back(byte);
  }
  return out;
}

std::string to_hex_string(const rust::Vec<uint8_t>& data) {
  static constexpr char kHexChars[] = "0123456789abcdef";
  std::string out;
  out.reserve(data.size() * 2 + 2);
  out.push_back('0');
  out.push_back('x');
  for (const auto byte : data) {
    out.push_back(kHexChars[(byte >> 4) & 0x0f]);
    out.push_back(kHexChars[byte & 0x0f]);
  }
  return out;
}

TWData* to_tw_data(const std::vector<uint8_t>& data) {
  return TWDataCreateWithBytes(data.data(), data.size());
}

TWData* to_tw_data(const rust::Vec<uint8_t>& data) {
  return TWDataCreateWithBytes(data.data(), data.size());
}

rust::Vec<uint8_t> to_rust_vec(const TWData* data) {
  rust::Vec<uint8_t> out;
  if (!data) {
    return out;
  }
  const auto* bytes = TWDataBytes(data);
  const auto size = TWDataSize(data);
  out.reserve(size);
  for (size_t i = 0; i < size; ++i) {
    out.push_back(bytes[i]);
  }
  return out;
}

TWString* to_tw_string(const rust::Str& value) {
  return TWStringCreateWithUTF8Bytes(std::string(value.data(), value.size()).c_str());
}

TWString* to_tw_string(const std::string& value) {
  return TWStringCreateWithUTF8Bytes(value.c_str());
}

SignerState* get_state(const WalletCoreSigner& signer) {
  return static_cast<SignerState*>(signer.inner);
}
}  // namespace

WalletCoreSigner::~WalletCoreSigner() {
  auto* state = static_cast<SignerState*>(inner);
  if (state && state->wallet) {
    TWHDWalletDelete(state->wallet);
    state->wallet = nullptr;
  }
  delete state;
  inner = nullptr;
}

std::unique_ptr<WalletCoreSigner> new_signer(const rust::Str& mnemonic,
                                             const rust::Str& passphrase) {
  auto signer = std::make_unique<WalletCoreSigner>();
  auto state = std::make_unique<SignerState>();
  TWString* mnemonic_str = to_tw_string(mnemonic);
  TWString* passphrase_str = to_tw_string(passphrase);
  if (!TWHDWalletIsValid(mnemonic_str)) {
    TWStringDelete(mnemonic_str);
    TWStringDelete(passphrase_str);
    return nullptr;
  }
  state->wallet = TWHDWalletCreateWithMnemonic(mnemonic_str, passphrase_str);
  TWStringDelete(mnemonic_str);
  TWStringDelete(passphrase_str);
  if (!state->wallet) {
    return nullptr;
  }
  signer->inner = state.release();
  return signer;
}

rust::Vec<std::uint8_t> derive_evm_address(const WalletCoreSigner& signer,
                                           const rust::Str& derivation_path) {
  auto* state = get_state(signer);
  if (!state || !state->wallet) {
    return {};
  }
  TWString* path_str = to_tw_string(derivation_path);
  TWDerivationPath* path = TWDerivationPathCreate(path_str);
  TWStringDelete(path_str);
  if (!path) {
    return {};
  }
  TWPrivateKey* private_key = TWHDWalletGetKey(state->wallet, path);
  TWDerivationPathDelete(path);
  if (!private_key) {
    return {};
  }
  TWPublicKey* public_key = TWPrivateKeyGetPublicKeySecp256k1(private_key);
  TWPrivateKeyDelete(private_key);
  if (!public_key) {
    return {};
  }
  TWEthereumAddress* address = TWEthereumAddressCreateWithPublicKey(public_key);
  TWPublicKeyDelete(public_key);
  if (!address) {
    return {};
  }
  TWData* address_data = TWEthereumAddressData(address);
  TWEthereumAddressDelete(address);
  auto out = to_rust_vec(address_data);
  TWDataDelete(address_data);
  return out;
}

rust::Vec<std::uint8_t> sign_eip1559(
    const WalletCoreSigner& signer,
    std::uint64_t chain_id,
    std::uint64_t nonce,
    const rust::Vec<std::uint8_t>& max_priority_fee_per_gas_be,
    const rust::Vec<std::uint8_t>& max_fee_per_gas_be,
    const rust::Vec<std::uint8_t>& gas_limit_be,
    const rust::Vec<std::uint8_t>& to20,
    const rust::Vec<std::uint8_t>& value_be,
    const rust::Vec<std::uint8_t>& data,
    const rust::Vec<std::uint8_t>& access_list_rlp) {
  auto* state = get_state(signer);
  if (!state || !state->wallet) {
    return {};
  }

  TWString* path_str = to_tw_string(kDefaultEvmDerivationPath);
  TWDerivationPath* path = TWDerivationPathCreate(path_str);
  TWStringDelete(path_str);
  if (!path) {
    return {};
  }
  TWPrivateKey* private_key = TWHDWalletGetKey(state->wallet, path);
  TWDerivationPathDelete(path);
  if (!private_key) {
    return {};
  }
  TWData* private_key_data = TWPrivateKeyData(private_key);
  TWPrivateKeyDelete(private_key);
  if (!private_key_data) {
    return {};
  }

  TW_Ethereum_Proto_SigningInput* input = TW_Ethereum_Proto_SigningInput_create();
  if (!input) {
    TWDataDelete(private_key_data);
    return {};
  }

  const auto chain_id_bytes = to_big_endian(chain_id);
  const auto nonce_bytes = to_big_endian(nonce);
  TWData* chain_id_data = to_tw_data(chain_id_bytes);
  TWData* nonce_data = to_tw_data(nonce_bytes);
  TWData* max_priority_fee_data = to_tw_data(max_priority_fee_per_gas_be);
  TWData* max_fee_data = to_tw_data(max_fee_per_gas_be);
  TWData* gas_limit_data = to_tw_data(gas_limit_be);
  TWData* amount_data = to_tw_data(value_be);
  TWData* payload_data = to_tw_data(data);
  TWData* access_list_data = to_tw_data(access_list_rlp);

  TW_Ethereum_Proto_SigningInput_set_chain_id(input, chain_id_data);
  TW_Ethereum_Proto_SigningInput_set_nonce(input, nonce_data);
  TW_Ethereum_Proto_SigningInput_set_max_inclusion_fee_per_gas(input,
                                                               max_priority_fee_data);
  TW_Ethereum_Proto_SigningInput_set_max_fee_per_gas(input, max_fee_data);
  TW_Ethereum_Proto_SigningInput_set_gas_limit(input, gas_limit_data);
  TW_Ethereum_Proto_SigningInput_set_amount(input, amount_data);
  TW_Ethereum_Proto_SigningInput_set_payload(input, payload_data);
  TW_Ethereum_Proto_SigningInput_set_private_key(input, private_key_data);
  TW_Ethereum_Proto_SigningInput_set_tx_mode(
      input, TW_Ethereum_Proto_TransactionMode_Enveloped);

  if (!to20.empty()) {
    const auto to_address_hex = to_hex_string(to20);
    TWString* to_address_str = to_tw_string(to_address_hex);
    TW_Ethereum_Proto_SigningInput_set_to_address(input, to_address_str);
    TWStringDelete(to_address_str);
  }

  if (!access_list_rlp.empty()) {
    TW_Ethereum_Proto_SigningInput_set_access_list(input, access_list_data);
  }

  TWDataDelete(chain_id_data);
  TWDataDelete(nonce_data);
  TWDataDelete(max_priority_fee_data);
  TWDataDelete(max_fee_data);
  TWDataDelete(gas_limit_data);
  TWDataDelete(amount_data);
  TWDataDelete(payload_data);
  TWDataDelete(access_list_data);

  TWData* input_data = TW_Ethereum_Proto_SigningInput_serialize(input);
  TW_Ethereum_Proto_SigningInput_delete(input);
  if (!input_data) {
    TWDataDelete(private_key_data);
    return {};
  }

  TWData* output_data = TWAnySignerSign(input_data, TWCoinTypeEthereum);
  TWDataDelete(input_data);
  TWDataDelete(private_key_data);
  if (!output_data) {
    return {};
  }

  TW_Ethereum_Proto_SigningOutput* output =
      TW_Ethereum_Proto_SigningOutput_deserialize(output_data);
  TWDataDelete(output_data);
  if (!output) {
    return {};
  }

  TWData* encoded = TW_Ethereum_Proto_SigningOutput_encoded(output);
  auto out = to_rust_vec(encoded);
  TWDataDelete(encoded);
  TW_Ethereum_Proto_SigningOutput_delete(output);
  return out;
}
