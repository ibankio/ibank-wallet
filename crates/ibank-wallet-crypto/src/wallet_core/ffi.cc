#include "ffi.h"

#include <string>
#include <vector>

#include <TrustWalletCore/AnySigner.h>
#include <TrustWalletCore/Coin.h>
#include <TrustWalletCore/DerivationPath.h>
#include <TrustWalletCore/HDWallet.h>
#include <TrustWalletCore/PrivateKey.h>
#include <TrustWalletCore/Ethereum/Address.h>
#include <TrustWalletCore/Ethereum/Proto.h>

namespace {
constexpr const char* kDefaultEvmDerivationPath = "m/44'/60'/0'/0/0";

struct SignerState {
  std::unique_ptr<TW::HDWallet> wallet;
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

std::string to_bytes_string(const rust::Vec<uint8_t>& data) {
  return std::string(reinterpret_cast<const char*>(data.data()), data.size());
}

std::string to_bytes_string(const std::vector<uint8_t>& data) {
  return std::string(reinterpret_cast<const char*>(data.data()), data.size());
}

rust::Vec<uint8_t> to_rust_vec(const TW::Data& data) {
  rust::Vec<uint8_t> out;
  out.reserve(data.size());
  for (const auto byte : data) {
    out.push_back(byte);
  }
  return out;
}

SignerState* get_state(const WalletCoreSigner& signer) {
  return static_cast<SignerState*>(signer.inner);
}
}  // namespace

WalletCoreSigner::~WalletCoreSigner() {
  auto* state = static_cast<SignerState*>(inner);
  delete state;
  inner = nullptr;
}

std::unique_ptr<WalletCoreSigner> new_signer(const rust::Str& mnemonic,
                                             const rust::Str& passphrase) {
  auto signer = std::make_unique<WalletCoreSigner>();
  auto state = std::make_unique<SignerState>();
  state->wallet = std::make_unique<TW::HDWallet>(
      std::string(mnemonic.data(), mnemonic.size()),
      std::string(passphrase.data(), passphrase.size()));
  if (!state->wallet || !state->wallet->isValid()) {
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
  TW::DerivationPath path(std::string(derivation_path.data(), derivation_path.size()));
  const auto key = state->wallet->getKey(path);
  const auto public_key = key.getPublicKey(TWPublicKeyTypeSECP256k1);
  const TW::Ethereum::Address address(public_key);
  return to_rust_vec(address.bytes);
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

  TW::DerivationPath path(kDefaultEvmDerivationPath);
  const auto private_key = state->wallet->getKey(path);

  TW::Ethereum::Proto::SigningInput input;
  const auto chain_id_bytes = to_big_endian(chain_id);
  const auto nonce_bytes = to_big_endian(nonce);
  input.set_chain_id(to_bytes_string(chain_id_bytes));
  input.set_nonce(to_bytes_string(nonce_bytes));
  input.set_max_inclusion_fee_per_gas(to_bytes_string(max_priority_fee_per_gas_be));
  input.set_max_fee_per_gas(to_bytes_string(max_fee_per_gas_be));
  input.set_gas_limit(to_bytes_string(gas_limit_be));
  input.set_amount(to_bytes_string(value_be));
  input.set_payload(to_bytes_string(data));
  input.set_private_key(private_key.bytes.data(), private_key.bytes.size());
  input.set_tx_mode(static_cast<TW::Ethereum::Proto::TransactionMode>(1));
  if (!to20.empty()) {
    const TW::Ethereum::Address address(TW::Data(to20.begin(), to20.end()));
    input.set_to_address(address.string());
  }
  if (!access_list_rlp.empty()) {
    input.set_access_list(to_bytes_string(access_list_rlp));
  }

  const auto serialized = input.SerializeAsString();
  const TW::Data input_data(serialized.begin(), serialized.end());
  const auto output_data = TW::AnySigner::sign(input_data, TWCoinTypeEthereum);
  TW::Ethereum::Proto::SigningOutput output;
  if (!output.ParseFromArray(output_data.data(), static_cast<int>(output_data.size()))) {
    return {};
  }
  return to_rust_vec(output.encoded());
}
