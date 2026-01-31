#pragma once
#include <cstdint>
#include <memory>
#include <vector>
#include "rust/cxx.h"

// Forward declare TW types if you want, but signer must be complete.
struct WalletCoreSigner {
  // Opaque pointer to internal state allocated in ffi.cc
  void* inner = nullptr;
  ~WalletCoreSigner();
};

// Constructor / destructor helpers
std::unique_ptr<WalletCoreSigner> new_signer(const rust::Str mnemonic,
                                             const rust::Str passphrase);

// EVM helpers
rust::Vec<std::uint8_t> derive_evm_address(const WalletCoreSigner& signer,
                                           const rust::Str derivation_path);

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
    const rust::Vec<std::uint8_t>& access_list_rlp);
