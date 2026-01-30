#include "ffi.h"

std::unique_ptr<WalletCoreSigner> new_signer(const rust::Str &mnemonic,
                                             const rust::Str &passphrase) {
  auto signer = std::make_unique<WalletCoreSigner>();
  signer->inner = nullptr; // later: store TWHDWallet* or your own state struct
  return signer;
}

std::vector<uint8_t> derive_evm_address(const WalletCoreSigner & /*signer*/,
                                        const rust::Str & /*derivation_path*/) {
  return std::vector<uint8_t>(20, 0);
}

std::vector<uint8_t> sign_eip1559(const WalletCoreSigner & /*signer*/,
                                  uint64_t /*chain_id*/,
                                  uint64_t /*nonce*/,
                                  const std::vector<uint8_t> & /*max_priority_fee_per_gas*/,
                                  const std::vector<uint8_t> & /*max_fee_per_gas*/,
                                  const std::vector<uint8_t> & /*gas_limit*/,
                                  const std::vector<uint8_t> & /*to*/,
                                  const std::vector<uint8_t> & /*value*/,
                                  const std::vector<uint8_t> & /*data*/,
                                  const std::vector<uint8_t> & /*access_list*/) {
  return {};
}
