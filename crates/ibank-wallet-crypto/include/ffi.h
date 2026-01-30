#pragma once

#include <cstdint>
#include <memory>
#include <string>
#include <vector>

class WalletCoreSigner;

std::unique_ptr<WalletCoreSigner> new_signer(const std::string &mnemonic,
                                             const std::string &passphrase);
std::vector<uint8_t> derive_evm_address(const WalletCoreSigner &signer,
                                        const std::string &derivation_path);
std::vector<uint8_t> sign_eip1559(const WalletCoreSigner &signer,
                                  uint64_t chain_id,
                                  uint64_t nonce,
                                  const std::vector<uint8_t> &max_priority_fee_per_gas,
                                  const std::vector<uint8_t> &max_fee_per_gas,
                                  const std::vector<uint8_t> &gas_limit,
                                  const std::vector<uint8_t> &to,
                                  const std::vector<uint8_t> &value,
                                  const std::vector<uint8_t> &data,
                                  const std::vector<uint8_t> &access_list);
