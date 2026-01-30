use std::path::{Path, PathBuf};

const WALLET_CORE_INCLUDE_ENV: &str = "WALLET_CORE_INCLUDE_DIR";
const WALLET_CORE_LIB_ENV: &str = "WALLET_CORE_LIB_DIR";
const PROTOBUF_PREFIX_ENV: &str = "PROTOBUF_PREFIX";
const PROTOBUF_LIB_ENV: &str = "PROTOBUF_LIB_DIR";
const FORCE_PROTOBUF_ENV: &str = "IBANK_WALLET_FORCE_PROTOBUF";
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
    println!("cargo:rerun-if-env-changed={}", MACOSX_DEPLOYMENT_TARGET_ENV);

    let wallet_core_include = find_wallet_core_include_dir();
    let wallet_core_lib = find_wallet_core_lib_dir();

    let mut build = cxx_build::bridge("src/wallet_core/ffi.rs");
    build.file("src/wallet_core/ffi.cc");
    build.flag_if_supported("-std=c++17");
    if let (Ok(target), Ok(os)) = (
        std::env::var(MACOSX_DEPLOYMENT_TARGET_ENV),
        std::env::var("CARGO_CFG_TARGET_OS"),
    ) {
        if os == "macos" {
            build.flag_if_supported(&format!("-mmacosx-version-min={}", target));
            println!(
                "cargo:rustc-link-arg=-Wl,-platform_version,macos,{0},{0}",
                target
            );
        }
    }

    build.include(wallet_core_include);
    build.include("include");
    build.include("src/wallet_core");
    build.compile("ibank_wallet_wallet_core");

    println!("cargo:rustc-link-search=native={}", wallet_core_lib.display());
    println!("cargo:rustc-link-lib=static=TrustWalletCore");

    if let Some((protobuf_lib_dir, protobuf_kind)) =
        resolve_protobuf_lib(&wallet_core_lib)
    {
        println!("cargo:rustc-link-search=native={}", protobuf_lib_dir.display());
        println!("cargo:rustc-link-lib={}=protobuf", protobuf_kind);
    }

    println!("cargo:rustc-link-lib=c++");
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

    let fallback = PathBuf::from("vendor/wallet-core/lib");
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
    if !wallet_core_needs_protobuf(wallet_core_lib) {
        // TrustWalletCore already bundles protobuf objects; avoid double-linking.
        return None;
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

    let nm_output = std::process::Command::new("nm")
        .arg("-gU")
        .arg(&lib_path)
        .output()
        .or_else(|_| {
            std::process::Command::new("nm")
                .arg("-g")
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
