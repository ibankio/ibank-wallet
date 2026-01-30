# ibank-wallet-crypto (wallet-core)

This crate can use TrustWallet Core via the `wallet-core` feature. On macOS arm64,
you must provide the TrustWallet Core headers and static library.

## Required environment variables

```bash
export WALLET_CORE_INCLUDE_DIR=/path/to/wallet-core/include
export WALLET_CORE_LIB_DIR=/path/to/wallet-core/lib
```

The build script falls back to `vendor/wallet-core/include` and
`vendor/wallet-core/lib` if they exist, but explicit paths are recommended for
deterministic builds.

## Protobuf linking (only if required by TrustWallet Core)

The build will inspect `libTrustWalletCore.a` for undefined protobuf symbols and
only link protobuf if needed. If detection fails or you want to force linking,
set:

```bash
export IBANK_WALLET_FORCE_PROTOBUF=1
```

You can also point to protobuf explicitly:

```bash
export PROTOBUF_PREFIX=/opt/homebrew/opt/protobuf
# or
export PROTOBUF_LIB_DIR=/opt/homebrew/opt/protobuf/lib
```

## macOS deployment target

To avoid mixed macOS minimum-version warnings, set a deployment target that
matches your toolchain and any prebuilt wallet-core artifacts:

```bash
export MACOSX_DEPLOYMENT_TARGET=11.0
```

## Build command

```bash
cargo run -p ibank-wallet-crypto --example evm --features wallet-core
```

## Debugging CXX bridge symbols

If you still see undefined symbols like `new_signer` or `derive_evm_address`,
verify they are present in the CXX bridge object:

```bash
nm -gU target/debug/build/ibank-wallet-crypto-*/out/*ffi*.o | rg "new_signer|derive_evm_address"
```
