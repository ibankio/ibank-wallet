#include "ffi.h"

#include <string>
#include <vector>

#include <TrustWalletCore/TWAnyAddress.h>
#include <TrustWalletCore/TWAnySigner.h>
#include <TrustWalletCore/TWCoinType.h>
#include <TrustWalletCore/TWData.h>
#include <TrustWalletCore/TWEthereum.h>
#include <TrustWalletCore/TWEthereumProto.h>
#include <TrustWalletCore/TWHDWallet.h>
#include <TrustWalletCore/TWPrivateKey.h>
#include <TrustWalletCore/TWPublicKey.h>
#include <TrustWalletCore/TWString.h>

#if __has_include(<TrustWalletCore/Proto/Ethereum.pb.h>)
#include <TrustWalletCore/Proto/Ethereum.pb.h>
#define IBANK_WALLET_HAS_ETHEREUM_PROTO 1
#elif __has_include(<TrustWalletCore/Ethereum/Proto/Ethereum.pb.h>)
#include <TrustWalletCore/Ethereum/Proto/Ethereum.pb.h>
#define IBANK_WALLET_HAS_ETHEREUM_PROTO 1
#else
#define IBANK_WALLET_HAS_ETHEREUM_PROTO 0
#endif

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

rust::Vec<uint8_t> to_rust_vec(const std::string& data) {
  rust::Vec<uint8_t> out;
  out.reserve(data.size());
  for (const auto byte : data) {
    out.push_back(static_cast<uint8_t>(byte));
  }
  return out;
}

std::string to_bytes_string(const std::vector<uint8_t>& data) {
  return std::string(reinterpret_cast<const char*>(data.data()), data.size());
}

std::string to_bytes_string(const rust::Vec<uint8_t>& data) {
  return std::string(reinterpret_cast<const char*>(data.data()), data.size());
}

TWString* to_tw_string(const rust::Str& value) {
  std::string buffer(value.data(), value.size());
  return TWStringCreateWithUTF8Bytes(buffer.c_str());
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
  if (!TWHDWalletMnemonicIsValid(mnemonic_str)) {
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
  TWPrivateKey* private_key = TWHDWalletGetKey(state->wallet, TWCoinTypeEthereum, path_str);
  TWStringDelete(path_str);
  if (!private_key) {
    return {};
  }
  TWPublicKey* public_key = TWPrivateKeyGetPublicKeySecp256k1(private_key, false);
  TWPrivateKeyDelete(private_key);
  if (!public_key) {
    return {};
  }
  TWAnyAddress* address = TWAnyAddressCreateWithPublicKey(public_key, TWCoinTypeEthereum);
  TWPublicKeyDelete(public_key);
  if (!address) {
    return {};
  }
  TWData* address_data = TWAnyAddressData(address);
  TWAnyAddressDelete(address);
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

  TWString* path_str = to_tw_string(std::string(kDefaultEvmDerivationPath));
  TWPrivateKey* private_key = TWHDWalletGetKey(state->wallet, TWCoinTypeEthereum, path_str);
  TWStringDelete(path_str);
  if (!private_key) {
    return {};
  }
  TWData* private_key_data = TWPrivateKeyData(private_key);
  TWPrivateKeyDelete(private_key);
  if (!private_key_data) {
    return {};
  }

  const auto chain_id_bytes = to_big_endian(chain_id);
  const auto nonce_bytes = to_big_endian(nonce);
#if IBANK_WALLET_HAS_ETHEREUM_PROTO
  TW::Ethereum::Proto::SigningInput input;
  input.set_chain_id(to_bytes_string(chain_id_bytes));
  input.set_nonce(to_bytes_string(nonce_bytes));
  input.set_max_inclusion_fee_per_gas(to_bytes_string(max_priority_fee_per_gas_be));
  input.set_max_fee_per_gas(to_bytes_string(max_fee_per_gas_be));
  input.set_gas_limit(to_bytes_string(gas_limit_be));
  input.set_amount(to_bytes_string(value_be));
  input.set_payload(to_bytes_string(data));
  input.set_private_key(
      std::string(reinterpret_cast<const char*>(TWDataBytes(private_key_data)),
                  TWDataSize(private_key_data)));
  input.set_tx_mode(TW::Ethereum::Proto::TransactionMode_Enveloped);

  if (!to20.empty()) {
    input.set_to_address(to_hex_string(to20));
  }

  if (!access_list_rlp.empty()) {
    input.set_access_list(to_bytes_string(access_list_rlp));
  }

  std::string serialized;
  if (!input.SerializeToString(&serialized)) {
    TWDataDelete(private_key_data);
    return {};
  }

  TWData* input_data =
      TWDataCreateWithBytes(reinterpret_cast<const uint8_t*>(serialized.data()),
                            serialized.size());
  TWData* output_data = TWAnySignerSign(input_data, TWCoinTypeEthereum);
  TWDataDelete(input_data);
  TWDataDelete(private_key_data);
  if (!output_data) {
    return {};
  }

  TW::Ethereum::Proto::SigningOutput output;
  output.ParseFromArray(TWDataBytes(output_data), TWDataSize(output_data));
  TWDataDelete(output_data);
  return to_rust_vec(output.encoded());
#else
  TWDataDelete(private_key_data);
  return {};
#endif
}
