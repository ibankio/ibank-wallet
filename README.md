# ibank-wallet

Rust-first wallet runtime + policy + chain adapters.
Crypto kernel: ibankio/wallet-core (vendored).

## Workspace layout

- `ibank-wallet-core`: core types, errors, audit log
- `ibank-wallet-crypto`: signer trait + wallet-core bridge
- `ibank-wallet-chains`: EVM types + EIP-1559 payload builder
- `ibank-wallet-policy`: policy engine skeleton
- `ibank-wallet-runtime`: intent -> quote -> policy -> sign -> submit orchestration

## Vendor wallet-core

```bash
git submodule add https://github.com/ibankio/wallet-core vendor/wallet-core
```

## Build wallet-core native library

> Replace the paths below with your local wallet-core build output.

### macOS

```bash
cd vendor/wallet-core
# Example build (adjust for your fork/toolchain)
# mkdir -p build && cd build
# cmake .. -DCMAKE_BUILD_TYPE=Release
# cmake --build . --target wallet_core
```

### Linux

```bash
cd vendor/wallet-core
# Example build (adjust for your fork/toolchain)
# mkdir -p build && cd build
# cmake .. -DCMAKE_BUILD_TYPE=Release
# cmake --build . --target wallet_core
```

## Build the Rust workspace

```bash
export WALLET_CORE_LIB_DIR=/path/to/wallet-core/build/lib
export WALLET_CORE_INCLUDE_DIR=/path/to/wallet-core/include

cargo build -p ibank-wallet-crypto
```

Required environment variables:

- `WALLET_CORE_INCLUDE_DIR`: path to wallet-core public headers (contains `TrustWalletCore/`).
- `WALLET_CORE_LIB_DIR`: path to the compiled static library output (contains `libTrustWalletCore.a`).

The Rust bridge links directly against the C++ wallet-core build products. You do **not** need to
build or open the TrustWalletCore Swift/Xcode workspace for this SDK integration.

If wallet-core is not available locally, disable the feature:

```bash
cargo build -p ibank-wallet-crypto --no-default-features
```

## Run tests

```bash
cargo test
```
