use std::path::{Path, PathBuf};

const WALLET_CORE_INCLUDE_ENV: &str = "WALLET_CORE_INCLUDE_DIR";
const WALLET_CORE_LIB_ENV: &str = "WALLET_CORE_LIB_DIR";
const PROTOBUF_PREFIX_ENV: &str = "PROTOBUF_PREFIX";
const PROTOBUF_LIB_ENV: &str = "PROTOBUF_LIB_DIR";
const FORCE_PROTOBUF_ENV: &str = "IBANK_WALLET_FORCE_PROTOBUF";
const FORCE_PROTOBUF_STATIC_ENV: &str = "IBANK_WALLET_FORCE_PROTOBUF_STATIC";
const SKIP_WALLET_CORE_SANITY_ENV: &str = "IBANK_WALLET_SKIP_WALLET_CORE_SANITY";
const MACOSX_DEPLOYMENT_TARGET_ENV: &str = "MACOSX_DEPLOYMENT_TARGET";

fn main() {
    if std::env::var("CARGO_FEATURE_WALLET_CORE").is_err() {
        return;
    }

    println!("cargo:rerun-if-changed=src/wallet_core/ffi.rs");
    println!("cargo:rerun-if-changed=src/wallet_core/ffi.cc");
    println!("cargo:rerun-if-changed=include/ffi.h");
    println!("cargo:rerun-if-env-changed={}", WALLET_CORE_INCLUDE_ENV);
    println!("cargo:rerun-if-env-changed={}", WALLET_CORE_LIB_ENV);
    println!("cargo:rerun-if-env-changed={}", PROTOBUF_PREFIX_ENV);
    println!("cargo:rerun-if-env-changed={}", PROTOBUF_LIB_ENV);
    println!("cargo:rerun-if-env-changed={}", FORCE_PROTOBUF_ENV);
    println!("cargo:rerun-if-env-changed={}", FORCE_PROTOBUF_STATIC_ENV);
    println!("cargo:rerun-if-env-changed={}", SKIP_WALLET_CORE_SANITY_ENV);
    println!("cargo:rerun-if-env-changed={}", MACOSX_DEPLOYMENT_TARGET_ENV);

    let wallet_core_include = find_wallet_core_include_dir();
    let wallet_core_lib = find_wallet_core_lib_dir();

    let mut build = cxx_build::bridge("src/wallet_core/ffi.rs");
    build.file("src/wallet_core/ffi.cc");
    build.flag_if_supported("-std=c++17");
    if std::env::var("CARGO_CFG_TARGET_OS").ok().as_deref() == Some("macos") {
        let target = std::env::var(MACOSX_DEPLOYMENT_TARGET_ENV).unwrap_or_else(|_| "11.0".to_string());
        build.flag_if_supported(&format!("-mmacosx-version-min={}", target));
        // Ensure the final link step uses the same deployment target.
        println!(
            "cargo:rustc-link-arg=-Wl,-platform_version,macos,{0},{0}",
            target
        );
    }

    build.include(wallet_core_include);
    build.include("include");
    build.include("src/wallet_core");
    build.compile("ibank_wallet_wallet_core");

    println!("cargo:rustc-link-search=native={}", wallet_core_lib.display());
    println!("cargo:rustc-link-lib=static=TrustWalletCore");

    // Link companion wallet-core archives when present (these satisfy symbols like
    // CURVE25519_NAME / tw_transaction_compiler_* / zil_schnorr_sign).
    // Different wallet-core build layouts may place these under subdirs such as trezor-crypto/.
    link_wallet_core_companion_archives(&wallet_core_lib);

    // Sanity-check: TrustWalletCore must not reference missing companion archives.
    // If the wallet-core build folder only contains libTrustWalletCore.a + libprotobuf.a,
    // but TrustWalletCore has undefined references to crypto/tx-compiler symbols, the
    // final binary link will fail (often only when building examples/bins).
    if std::env::var(SKIP_WALLET_CORE_SANITY_ENV).is_ok() {
        println!(
            "cargo:warning={SKIP_WALLET_CORE_SANITY_ENV} is set; skipping TrustWalletCore self-containment checks. \n\
           If you hit undefined symbols at link time, rebuild wallet-core to bundle companion libs or link the missing .a archives."
        );
    } else {
        ensure_wallet_core_is_self_contained(&wallet_core_lib);
    }

    if let Some((protobuf_lib_dir, protobuf_kind)) =
        resolve_protobuf_lib(&wallet_core_lib)
    {
        println!("cargo:rustc-link-search=native={}", protobuf_lib_dir.display());
        println!("cargo:rustc-link-lib={}=protobuf", protobuf_kind);
    }

    // C++ stdlib (required for wallet-core)
    println!("cargo:rustc-link-lib=c++");

    // Common system libs used by wallet-core on macOS (safe no-ops if unused)
    if std::env::var("CARGO_CFG_TARGET_OS").ok().as_deref() == Some("macos") {
        println!("cargo:rustc-link-lib=framework=Security");
        println!("cargo:rustc-link-lib=framework=Foundation");
        println!("cargo:rustc-link-lib=z");
        println!("cargo:rustc-link-lib=bz2");
    }
}

fn find_wallet_core_include_dir() -> PathBuf {
    if let Ok(dir) = std::env::var(WALLET_CORE_INCLUDE_ENV) {
        let path = PathBuf::from(dir);
        if path.is_dir() {
            return path;
        }
        panic!(
            "{WALLET_CORE_INCLUDE_ENV} is set but does not exist: {}",
            path.display()
        );
    }

    let fallback = PathBuf::from("vendor/wallet-core/include");
    if fallback.is_dir() {
        println!(
            "cargo:warning=Using wallet-core headers from {} (set {WALLET_CORE_INCLUDE_ENV} to override).",
            fallback.display()
        );
        return fallback;
    }

    panic!(
        "Missing wallet-core headers. Set {WALLET_CORE_INCLUDE_ENV} to the TrustWalletCore include directory."
    );
}

fn find_wallet_core_lib_dir() -> PathBuf {
    if let Ok(dir) = std::env::var(WALLET_CORE_LIB_ENV) {
        let path = PathBuf::from(dir);
        if path.is_dir() {
            return path;
        }
        panic!(
            "{WALLET_CORE_LIB_ENV} is set but does not exist: {}",
            path.display()
        );
    }

    let fallback = PathBuf::from("vendor/wallet-core/build");
    if fallback.is_dir() {
        println!(
            "cargo:warning=Using wallet-core libs from {} (set {WALLET_CORE_LIB_ENV} to override).",
            fallback.display()
        );
        return fallback;
    }

    panic!(
        "Missing wallet-core libs. Set {WALLET_CORE_LIB_ENV} to the directory containing libTrustWalletCore.a."
    );
}

fn resolve_protobuf_lib(wallet_core_lib: &Path) -> Option<(PathBuf, &'static str)> {
    // If the wallet-core build provides a bundled static protobuf, prefer it.
    // This avoids mixing Homebrew protobuf with wallet-core's compiled objects.
    if std::env::var(FORCE_PROTOBUF_STATIC_ENV).is_ok() {
        let bundled = wallet_core_lib.join("libprotobuf.a");
        if bundled.is_file() {
            return Some((wallet_core_lib.to_path_buf(), "static"));
        }
        println!(
            "cargo:warning={FORCE_PROTOBUF_STATIC_ENV} is set but libprotobuf.a was not found in {}",
            wallet_core_lib.display()
        );
    }

    if !wallet_core_needs_protobuf(wallet_core_lib) {
        // TrustWalletCore already bundles protobuf objects; avoid double-linking.
        return None;
    }

    // If wallet-core ships libprotobuf.a, link it as static.
    let bundled = wallet_core_lib.join("libprotobuf.a");
    if bundled.is_file() {
        return Some((wallet_core_lib.to_path_buf(), "static"));
    }

    if std::env::var(FORCE_PROTOBUF_ENV).is_ok() {
        return find_protobuf_dir();
    }

    match find_protobuf_dir() {
        Some(found) => Some(found),
        None => {
            panic!(
                "TrustWalletCore references protobuf but no protobuf library could be found. \
Set {PROTOBUF_PREFIX_ENV} or {PROTOBUF_LIB_ENV}, or install protobuf (brew install protobuf)."
            );
        }
    }
}

fn wallet_core_needs_protobuf(wallet_core_lib: &Path) -> bool {
    let lib_path = wallet_core_lib.join("libTrustWalletCore.a");
    if !lib_path.is_file() {
        println!(
            "cargo:warning=libTrustWalletCore.a not found at {} (skipping protobuf detection).",
            lib_path.display()
        );
        return false;
    }

    // Prefer `nm -u` (undefined symbols) so we can reliably detect external protobuf deps.
    let nm_output = std::process::Command::new("nm")
        .arg("-u")
        .arg(&lib_path)
        .output()
        .or_else(|_| {
            std::process::Command::new("nm")
                .arg("-u")
                .arg(&lib_path)
                .output()
        });

    let output = match nm_output {
        Ok(output) if output.status.success() => output.stdout,
        _ => {
            println!(
                "cargo:warning=Unable to run nm for protobuf detection. Set {FORCE_PROTOBUF_ENV}=1 to link protobuf explicitly."
            );
            return false;
        }
    };

    let output = String::from_utf8_lossy(&output);
    output.lines().any(|line| {
        line.contains("google::protobuf") && (line.contains(" U ") || line.starts_with("U "))
    })
}

fn find_protobuf_dir() -> Option<(PathBuf, &'static str)> {
    if let Ok(dir) = std::env::var(PROTOBUF_LIB_ENV) {
        let path = PathBuf::from(dir);
        if path.is_dir() {
            return Some((path, "dylib"));
        }
        panic!(
            "{PROTOBUF_LIB_ENV} is set but does not exist: {}",
            path.display()
        );
    }

    if let Ok(prefix) = std::env::var(PROTOBUF_PREFIX_ENV) {
        let path = PathBuf::from(prefix);
        let lib_path = path.join("lib");
        if lib_path.is_dir() {
            return Some((lib_path, "dylib"));
        }
        panic!(
            "{PROTOBUF_PREFIX_ENV} is set but does not contain lib/: {}",
            path.display()
        );
    }

    if let Ok(output) = std::process::Command::new("brew")
        .args(["--prefix", "protobuf"])
        .output()
    {
        if output.status.success() {
            let prefix = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !prefix.is_empty() {
                let lib_path = PathBuf::from(prefix).join("lib");
                if lib_path.is_dir() {
                    return Some((lib_path, "dylib"));
                }
            }
        }
    }

    for prefix in ["/opt/homebrew/opt/protobuf", "/usr/local/opt/protobuf"] {
        let lib_path = Path::new(prefix).join("lib");
        if lib_path.is_dir() {
            return Some((lib_path, "dylib"));
        }
    }

    None
}

fn ensure_wallet_core_is_self_contained(wallet_core_lib: &Path) {
    let lib_path = wallet_core_lib.join("libTrustWalletCore.a");
    if !lib_path.is_file() {
        return;
    }

    // Detect a few known missing components that often appear when wallet-core was built
    // without bundling companion static archives (e.g. TrezorCrypto / tx-compiler / zil).
    let nm_output = {
        let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
        let mut command = std::process::Command::new("nm");
        if target_os == "macos" {
            command.arg("-gU");
        } else {
            command.args(["-g", "-u"]);
        }
        command.arg(&lib_path).output()
    };

    let output = match nm_output {
        Ok(output) if output.status.success() => String::from_utf8_lossy(&output.stdout).to_string(),
        _ => {
            println!(
                "cargo:warning=Unable to run nm to validate TrustWalletCore self-containment. Skipping."
            );
            return;
        }
    };

    let suspects = [
        "CURVE25519_NAME",
        "ED25519_NAME",
        "ED25519_CARDANO_NAME",
        "ED25519_BLAKE2B_NANO_NAME",
        "NIST256P1_NAME",
        "tw_transaction_compiler_compile",
        "tw_transaction_compiler_pre_image_hashes",
        "zil_schnorr_sign",
    ];

    let mut found = Vec::new();
    for s in suspects {
        if output.contains(s) {
            found.push(s);
        }
    }

    if found.is_empty() {
        return;
    }

    // If wallet-core only produced TrustWalletCore + protobuf, but has undefined symbols for
    // these components, the final link will fail. Provide a clear actionable error.
    let protobuf_a = wallet_core_lib.join("libprotobuf.a");

    // Count only top-level .a files (legacy behavior) ...
    let top_level_a_count = {
        let mut count = 0usize;
        if let Ok(entries) = std::fs::read_dir(wallet_core_lib) {
            for e in entries.flatten() {
                if e.path().extension().and_then(|s| s.to_str()) == Some("a") {
                    count += 1;
                }
            }
        }
        count
    };

    // ...but also consider common subdirs where wallet-core places companion archives.
    let has_companions_in_subdirs = wallet_core_lib.join("trezor-crypto").join("libTrezorCrypto.a").is_file()
        || wallet_core_lib.join("walletconsole").join("lib").join("libwalletconsolelib.a").is_file()
        || wallet_core_lib.join("local").join("lib").join("libwallet_core_rs.a").is_file();

    let has_only_two_archives = top_level_a_count <= 2 && protobuf_a.is_file() && !has_companions_in_subdirs;

    if has_only_two_archives {
        panic!(
            "TrustWalletCore has undefined references to: {:?}. \
Your wallet-core build directory ({}) appears to only contain libTrustWalletCore.a and libprotobuf.a. \
This usually means wallet-core was built without bundling companion static libs (crypto/tx-compiler). \
Rebuild wallet-core so these objects are included in libTrustWalletCore.a or additional .a archives are produced, then link them from build.rs. \
\
To bypass this check temporarily (not recommended), set {SKIP_WALLET_CORE_SANITY_ENV}=1.",
            found,
            wallet_core_lib.display()
        );
    } else {
        println!(
            "cargo:warning=TrustWalletCore has undefined references to {:?}. Ensure all required wallet-core companion archives are present in {} and linked.",
            found,
            wallet_core_lib.display()
        );
    }
}

fn link_wallet_core_companion_archives(wallet_core_lib: &Path) {
    // Add search paths for common wallet-core subdirectories that may contain companion archives.
    // Keep these deterministic and cheap; don't recurse arbitrarily.
    let candidates: Vec<PathBuf> = vec![
        wallet_core_lib.to_path_buf(),
        wallet_core_lib.join("trezor-crypto"),
        wallet_core_lib.join("walletconsole").join("lib"),
        wallet_core_lib.join("local").join("lib"),
        wallet_core_lib.join("lib"),
    ];

    for dir in candidates.iter() {
        if dir.is_dir() {
            println!("cargo:rustc-link-search=native={}", dir.display());
        }
    }

    // Link the key companion archives if present. We use filenames to decide what to link.
    // Note: `rustc-link-lib=static=<name>` expects lib<name>.a
    let mut linked = false;

    // trezor-crypto provides curve constants & various crypto primitives
    if wallet_core_lib.join("trezor-crypto").join("libTrezorCrypto.a").is_file()
        || wallet_core_lib.join("libTrezorCrypto.a").is_file()
    {
        println!("cargo:rustc-link-lib=static=TrezorCrypto");
        linked = true;
    }

    // TransactionCompiler symbols may be built into TrustWalletCore, but some builds produce a separate archive.
    // Link it when it exists to satisfy tw_transaction_compiler_* references.
    if wallet_core_lib.join("libTransactionCompiler.a").is_file()
        || wallet_core_lib.join("transaction-compiler").join("libTransactionCompiler.a").is_file()
        || wallet_core_lib.join("walletconsole").join("lib").join("libTransactionCompiler.a").is_file()
        || wallet_core_lib.join("local").join("lib").join("libTransactionCompiler.a").is_file()
    {
        println!("cargo:rustc-link-lib=static=TransactionCompiler");
        linked = true;
    }

    // Some builds generate a consolidated Rust helper archive.
    if wallet_core_lib.join("local").join("lib").join("libwallet_core_rs.a").is_file()
        || wallet_core_lib.join("libwallet_core_rs.a").is_file()
    {
        println!("cargo:rustc-link-lib=static=wallet_core_rs");
        linked = true;
    }

    // walletconsolelib is not required for core signing, but some builds park common objects there.
    // Link it opportunistically if present (and only then).
    if wallet_core_lib.join("walletconsole").join("lib").join("libwalletconsolelib.a").is_file()
        || wallet_core_lib.join("libwalletconsolelib.a").is_file()
    {
        println!("cargo:rustc-link-lib=static=walletconsolelib");
        linked = true;
    }

    if linked {
        println!("cargo:warning=Linked wallet-core companion archives (TrezorCrypto / TransactionCompiler / wallet_core_rs / walletconsolelib) when present.");
    } else {
        println!(
            "cargo:warning=No wallet-core companion archives detected under {}. If you see undefined symbols (CURVE25519_NAME / tw_transaction_compiler_* / zil_schnorr_sign), rebuild wallet-core and ensure companion .a files are available.",
            wallet_core_lib.display()
        );
    }
}

