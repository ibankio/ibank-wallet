fn main() {
    if std::env::var("CARGO_FEATURE_WALLET_CORE").is_err() {
        return;
    }

    println!("cargo:rerun-if-changed=src/wallet_core/ffi.rs");
    println!("cargo:rerun-if-changed=src/wallet_core/ffi.cc");
    println!("cargo:rerun-if-env-changed=WALLET_CORE_INCLUDE_DIR");
    println!("cargo:rerun-if-env-changed=WALLET_CORE_LIB_DIR");

    let mut build = cxx_build::bridge("src/wallet_core/ffi.rs");
    build.file("src/wallet_core/ffi.cc");
    build.flag_if_supported("-std=c++17");

    // 1) Wallet-core public headers (contains TrustWalletCore/*.h)
    //    You already set this to: vendor/wallet-core/include
    if let Ok(include_dir) = std::env::var("WALLET_CORE_INCLUDE_DIR") {
        build.include(include_dir);
    }

    // 2) Our local shim headers (if you have any alongside ffi.h)

    // IMPORTANT: where your ffi.h lives
    build.include("include");       // <-- this makes #include "ffi.h" work

    //    This is relative to crates/ibank-wallet-crypto/
    build.include("src/wallet_core");

    build.compile("ibank_wallet_wallet_core");

    // Link wallet-core static lib
    if let Ok(lib_dir) = std::env::var("WALLET_CORE_LIB_DIR") {
        println!("cargo:rustc-link-search=native={}", lib_dir);
    }

    // You built: libTrustWalletCore.a
    println!("cargo:rustc-link-lib=static=TrustWalletCore");
}