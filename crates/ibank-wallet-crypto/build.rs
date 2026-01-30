fn main() {
    if std::env::var("CARGO_FEATURE_WALLET_CORE").is_err() {
        return;
    }

    let mut build = cxx_build::bridge("src/wallet_core/ffi.rs");
    build.file("src/wallet_core/ffi.cc");

    if let Ok(include_dir) = std::env::var("WALLET_CORE_INCLUDE_DIR") {
        build.include(include_dir);
    }

    build.include("include");
    build.flag_if_supported("-std=c++17");

    build.compile("ibank_wallet_wallet_core");

    if let Ok(lib_dir) = std::env::var("WALLET_CORE_LIB_DIR") {
        println!("cargo:rustc-link-search=native={}", lib_dir);
    }
    println!("cargo:rustc-link-lib=static=wallet_core");
}
