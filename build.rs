#![allow(clippy::disallowed_methods)]

use std::process::Command;

fn main() {
    if let Err(error) = try_main() {
        println!(
            "cargo::warning=aimx: build script setup failed: {error}. \
             All APIs will return Err(Error::Unavailable) at runtime."
        );
    }
}

fn try_main() -> Result<(), BuildScriptError> {
    let target_os = match std::env::var("CARGO_CFG_TARGET_OS") {
        Ok(value) => value,
        Err(_) => return Ok(()),
    };

    if target_os != "macos" {
        return Ok(());
    }

    let manifest_dir = cargo_env("CARGO_MANIFEST_DIR")?;
    let out_dir = cargo_env("OUT_DIR")?;
    let swift_file = format!("{manifest_dir}/bridge.swift");
    let lib_path = format!("{out_dir}/libaimx_bridge.a");

    println!("cargo:rerun-if-changed=bridge.swift");
    // Declare the custom cfg flag so rustc doesn't warn about it on any host.
    println!("cargo::rustc-check-cfg=cfg(aimx_bridge)");

    let status = Command::new("xcrun")
        .args([
            "swiftc",
            "-emit-library",
            "-static",
            "-parse-as-library",
            "-module-name",
            "AIMXBridge",
            // Minimum deployment target; FoundationModels is guarded at runtime with
            // #available so the binary still runs on older macOS without crashing.
            "-target",
            "arm64-apple-macos15.0",
            "-o",
            &lib_path,
            &swift_file,
        ])
        .status();

    match status {
        Ok(s) if s.success() => {
            println!("cargo:rustc-cfg=aimx_bridge");
            println!("cargo:rustc-link-search=native={out_dir}");
            println!("cargo:rustc-link-lib=static=aimx_bridge");
            // FoundationModels is weak-linked so the binary doesn't crash on
            // macOS < 26 where the framework doesn't exist yet.
            println!("cargo:rustc-link-arg=-Wl,-weak_framework,FoundationModels");
            // Required for Swift concurrency runtime.
            println!("cargo:rustc-link-arg=-Wl,-rpath,/usr/lib/swift");
        }
        Ok(_) => {
            println!(
                "cargo::warning=aimx: Swift bridge compilation failed \
                 (requires Xcode with macOS 26+ SDK). All APIs will return \
                 Err(Error::Unavailable) at runtime."
            );
        }
        Err(e) => {
            println!(
                "cargo::warning=aimx: xcrun not found, \
                 skipping Swift bridge: {e}"
            );
        }
    }

    Ok(())
}

fn cargo_env(name: &'static str) -> Result<String, BuildScriptError> {
    std::env::var(name).map_err(|source| BuildScriptError::MissingCargoEnv { name, source })
}

#[derive(Debug, thiserror::Error)]
enum BuildScriptError {
    #[error("required Cargo environment variable {name} is unavailable")]
    MissingCargoEnv {
        name: &'static str,
        #[source]
        source: std::env::VarError,
    },
}
