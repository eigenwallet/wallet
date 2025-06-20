use cmake::Config;
use std::fs;
use std::path::Path;

/// Represents a patch to be applied to the Monero codebase
struct EmbeddedPatch {
    name: &'static str,
    description: &'static str,
    patch_unified: &'static str,
}

/// Macro to create embedded patches with compile-time file inclusion
macro_rules! embedded_patch {
    ($name:literal, $description:literal, $patch_file:literal) => {
        EmbeddedPatch {
            name: $name,
            description: $description,
            patch_unified: include_str!($patch_file),
        }
    };
}

/// Embedded patches applied at compile time
const EMBEDDED_PATCHES: &[EmbeddedPatch] = &[embedded_patch!(
    "wallet2_api_allow_subtract_from_fee",
    "Adds subtract_fee_from_outputs parameter to wallet2_api transaction creation methods",
    "patches/wallet2_api_allow_subtract_from_fee.patch"
)];

fn apply_embedded_patches() -> Result<(), Box<dyn std::error::Error>> {
    let monero_dir = Path::new("monero");

    if !monero_dir.exists() {
        return Err("Monero directory not found. Please ensure the monero submodule is initialized and present.".into());
    }

    for embedded in EMBEDDED_PATCHES {
        println!(
            "cargo:warning=Applying embedded patch: {} ({}) with content: {}",
            embedded.name, embedded.description, embedded.patch_unified
        );

        // Try parsing the entire patch first
        let patch = diffy::Patch::from_str(embedded.patch_unified)
            .map_err(|e| format!("Failed to parse patch {}: {}", embedded.name, e))?;

        // Get the file path from patch headers
        let raw_path = patch
            .modified()
            .or_else(|| patch.original())
            .ok_or_else(|| format!("Patch {} does not specify a file", embedded.name))?;

        let clean_path = raw_path
            .strip_prefix("a/")
            .or_else(|| raw_path.strip_prefix("b/"))
            .expect("Failed to strip prefix from Monero patch");

        let target_path = monero_dir.join(clean_path);

        if !target_path.exists() {
            return Err(format!("Target file {} not found!", clean_path).into());
        }

        let current = fs::read_to_string(&target_path)
            .map_err(|e| format!("Failed to read {}: {}", clean_path, e))?;

        let patched = match diffy::apply(&current, &patch) {
            Ok(p) => p,
            Err(_) => {
                // Try reversing the patch – if that succeeds the file already contains the changes
                if let Ok(_) = diffy::apply(&current, &patch.reverse()) {
                    println!(
                        "cargo:warning=Patch {} already applied to {} – skipping",
                        embedded.name, clean_path
                    );
                    continue;
                } else {
                    return Err(format!(
                        "Failed to apply patch {} to {}: hunk mismatch (not already applied)",
                        embedded.name, clean_path
                    )
                    .into());
                }
            }
        };

        fs::write(&target_path, patched)
            .map_err(|e| format!("Failed to write {}: {}", clean_path, e))?;

        println!(
            "cargo:warning=Successfully applied embedded patch: {} ({}).",
            embedded.name, embedded.description
        );
    }

    Ok(())
}

fn main() {
    let is_github_actions: bool = std::env::var("GITHUB_ACTIONS").is_ok();
    let is_docker_build: bool = std::env::var("DOCKER_BUILD").is_ok();

    // Eerun this when the bridge.rs or static_bridge.h file changes.
    println!("cargo:rerun-if-changed=src/bridge.rs");
    println!("cargo:rerun-if-changed=src/bridge.h");

    // Rerun if this build script changes (since it contains embedded patches)
    println!("cargo:rerun-if-changed=build.rs");

    // Rerun if the patches directory or any patch files change
    println!("cargo:rerun-if-changed=patches");

    // Apply embedded patches before building
    if let Err(e) = apply_embedded_patches() {
        panic!("Failed to apply embedded patches: {}", e);
    }

    // Build with the monero library all dependencies required
    let mut config = Config::new("monero");

    let output_directory = config
        .build_target("wallet_api")
        // Builds currently fail in Release mode
        // .define("CMAKE_BUILD_TYPE", "Release")
        // .define("CMAKE_RELEASE_TYPE", "Release")
        // Force building static libraries
        .define("STATIC", "ON")
        .define("BUILD_SHARED_LIBS", "OFF")
        .define("BUILD_TESTS", "OFF")
        .define("Boost_USE_STATIC_LIBS", "ON")
        .define("Boost_USE_STATIC_RUNTIME", "ON")
        //// Disable support for ALL hardware wallets
        // Disable Trezor support completely
        .define("USE_DEVICE_TREZOR", "OFF")
        .define("USE_DEVICE_TREZOR_MANDATORY", "OFF")
        .define("USE_DEVICE_TREZOR_PROTOBUF_TEST", "OFF")
        .define("USE_DEVICE_TREZOR_LIBUSB", "OFF")
        .define("USE_DEVICE_TREZOR_UDP_RELEASE", "OFF")
        .define("USE_DEVICE_TREZOR_DEBUG", "OFF")
        .define("TREZOR_DEBUG", "OFF")
        // Prevent CMake from finding dependencies that could enable Trezor
        .define("CMAKE_DISABLE_FIND_PACKAGE_LibUSB", "ON")
        // Disable Ledger support
        .define("USE_DEVICE_LEDGER", "OFF")
        .define("CMAKE_DISABLE_FIND_PACKAGE_HIDAPI", "ON")
        .define("GTEST_HAS_ABSL", "OFF")
        // Use lightweight crypto library
        .define("MONERO_WALLET_CRYPTO_LIBRARY", "cn")
        .build_arg("-Wno-dev") // Disable warnings we can't fix anyway
        .build_arg(match (is_github_actions, is_docker_build) {
            (true, _) => "-j1",
            (_, true) => "-j1",
            (_, _) => "-j",
        })
        .build();

    let monero_build_dir = output_directory.join("build");

    println!(
        "cargo:debug=Build directory: {}",
        output_directory.display()
    );

    // Add output directories to the link search path
    println!(
        "cargo:rustc-link-search=native={}",
        monero_build_dir.join("lib").display()
    );

    // Add additional link search paths for libraries in different directories
    println!(
        "cargo:rustc-link-search=native={}",
        monero_build_dir.join("contrib/epee/src").display()
    );
    println!(
        "cargo:rustc-link-search=native={}",
        monero_build_dir.join("external/easylogging++").display()
    );
    println!(
        "cargo:rustc-link-search=native={}",
        monero_build_dir
            .join("external/db_drivers/liblmdb")
            .display()
    );
    println!(
        "cargo:rustc-link-search=native={}",
        monero_build_dir.join("external/randomx").display()
    );
    println!("cargo:rustc-link-search=native=/usr/lib/x86_64-linux-gnu");

    println!(
        "cargo:rustc-link-search=native={}",
        monero_build_dir.join("src").display()
    );
    println!(
        "cargo:rustc-link-search=native={}",
        monero_build_dir.join("src/crypto").display()
    );
    println!(
        "cargo:rustc-link-search=native={}",
        monero_build_dir.join("src/net").display()
    );
    println!(
        "cargo:rustc-link-search=native={}",
        monero_build_dir.join("src/ringct").display()
    );
    println!(
        "cargo:rustc-link-search=native={}",
        monero_build_dir.join("src/checkpoints").display()
    );
    println!(
        "cargo:rustc-link-search=native={}",
        monero_build_dir.join("src/multisig").display()
    );
    println!(
        "cargo:rustc-link-search=native={}",
        monero_build_dir.join("src/cryptonote_basic").display()
    );
    println!(
        "cargo:rustc-link-search=native={}",
        monero_build_dir.join("src/common").display()
    );
    println!(
        "cargo:rustc-link-search=native={}",
        monero_build_dir.join("src/cryptonote_core").display()
    );
    println!(
        "cargo:rustc-link-search=native={}",
        monero_build_dir.join("src/hardforks").display()
    );
    println!(
        "cargo:rustc-link-search=native={}",
        monero_build_dir.join("src/blockchain_db").display()
    );
    println!(
        "cargo:rustc-link-search=native={}",
        monero_build_dir.join("src/device").display()
    );
    // device_trezor search path (stub version when disabled)
    println!(
        "cargo:rustc-link-search=native={}",
        monero_build_dir.join("src/device_trezor").display()
    );
    println!(
        "cargo:rustc-link-search=native={}",
        monero_build_dir.join("src/mnemonics").display()
    );
    println!(
        "cargo:rustc-link-search=native={}",
        monero_build_dir.join("src/rpc").display()
    );

    #[cfg(target_os = "macos")]
    {
        // Dynamically detect Homebrew installation prefix (works on both Apple Silicon and Intel Macs)
        let brew_prefix = std::process::Command::new("brew")
            .arg("--prefix")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "/opt/homebrew".into());

        // add homebrew search paths using dynamic prefix
        println!("cargo:rustc-link-search=native={}/lib", brew_prefix);
        println!(
            "cargo:rustc-link-search=native={}/opt/unbound/lib",
            brew_prefix
        );
        println!(
            "cargo:rustc-link-search=native={}/opt/expat/lib",
            brew_prefix
        );
        println!(
            "cargo:rustc-link-search=native={}/Cellar/protobuf@21/21.12_1/lib/",
            brew_prefix
        );

        // Add search paths for clang runtime libraries
        println !("cargo:rustc-link-search=native=/Library/Developer/CommandLineTools/usr/lib/clang/15.0.0/lib/darwin");
        println !("cargo:rustc-link-search=native=/Library/Developer/CommandLineTools/usr/lib/clang/16.0.0/lib/darwin");
        println !("cargo:rustc-link-search=native=/Library/Developer/CommandLineTools/usr/lib/clang/17.0.0/lib/darwin");
        println !("cargo:rustc-link-search=native=/Library/Developer/CommandLineTools/usr/lib/clang/18.0.0/lib/darwin");
    }

    // Link libwallet and libwallet_api statically
    println!("cargo:rustc-link-lib=static=wallet");
    println!("cargo:rustc-link-lib=static=wallet_api");

    // Link targets of monero codebase statically
    println!("cargo:rustc-link-lib=static=epee");
    println!("cargo:rustc-link-lib=static=easylogging");
    println!("cargo:rustc-link-lib=static=lmdb");
    println!("cargo:rustc-link-lib=static=randomx");
    println!("cargo:rustc-link-lib=static=cncrypto");
    println!("cargo:rustc-link-lib=static=net");
    println!("cargo:rustc-link-lib=static=ringct");
    println!("cargo:rustc-link-lib=static=ringct_basic");
    println!("cargo:rustc-link-lib=static=checkpoints");
    println!("cargo:rustc-link-lib=static=multisig");
    println!("cargo:rustc-link-lib=static=version");
    println!("cargo:rustc-link-lib=static=cryptonote_basic");
    println!("cargo:rustc-link-lib=static=cryptonote_format_utils_basic");
    println!("cargo:rustc-link-lib=static=common");
    println!("cargo:rustc-link-lib=static=cryptonote_core");
    println!("cargo:rustc-link-lib=static=hardforks");
    println!("cargo:rustc-link-lib=static=blockchain_db");
    println!("cargo:rustc-link-lib=static=device");
    // Link device_trezor (stub version when USE_DEVICE_TREZOR=OFF)
    println!("cargo:rustc-link-lib=static=device_trezor");
    println!("cargo:rustc-link-lib=static=mnemonics");
    println!("cargo:rustc-link-lib=static=rpc_base");

    // Static linking for boost
    println!("cargo:rustc-link-lib=static=boost_serialization");
    println!("cargo:rustc-link-lib=static=boost_filesystem");
    println!("cargo:rustc-link-lib=static=boost_thread");
    println!("cargo:rustc-link-lib=static=boost_chrono");

    // Link libsodium statically
    println!("cargo:rustc-link-lib=static=sodium");

    // Link OpenSSL statically
    println!("cargo:rustc-link-lib=static=ssl"); // This is OpenSSL (libsll)
    println!("cargo:rustc-link-lib=static=crypto"); // This is OpenSSLs crypto library (libcrypto)

    // Link protobuf statically
    println!("cargo:rustc-link-lib=static=protobuf");

    #[cfg(target_os = "macos")]
    {
        // Static archive is always present, dylib only on some versions.
        println!("cargo:rustc-link-lib=static=clang_rt.osx");

        // Minimum OS version you already add:
        println!("cargo:rustc-link-arg=-mmacosx-version-min=11.0");
    }

    // Build the CXX bridge
    let mut build = cxx_build::bridge("src/bridge.rs");

    #[cfg(target_os = "macos")]
    {
        build.flag_if_supported("-mmacosx-version-min=11.0");
    }

    build
        .flag_if_supported("-std=c++17")
        .include("src") // Include the bridge.h file
        .include("monero/src") // Includes the monero headers
        .include("monero/external/easylogging++") // Includes the easylogging++ headers
        .include("monero/contrib/epee/include") // Includes the epee headers for net/http_client.h
        .include("/opt/homebrew/include") // Homebrew include path for Boost
        .flag("-fPIC"); // Position independent code

    #[cfg(target_os = "macos")]
    {
        // Use the same dynamic brew prefix for include paths
        let brew_prefix = std::process::Command::new("brew")
            .arg("--prefix")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "/opt/homebrew".into());

        build.include(format!("{}/include", brew_prefix)); // Homebrew include path for Boost
    }

    build.compile("monero-sys");
}
