#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::arithmetic_side_effects,
    clippy::collapsible_if,
    clippy::manual_assert
)]
use std::process::Command;

fn needs_build(src: &[String], dst: &str) -> bool {
    let Ok(wasm_meta) = std::fs::metadata(dst).and_then(|m| m.modified()) else {
        return true;
    };
    for s in src {
        if let Ok(meta) = std::fs::metadata(s).and_then(|m| m.modified()) {
            if meta > wasm_meta {
                return true;
            }
        }
    }
    false
}

fn build_one(manifest: &str, wasm_name: &str, out: &str, target_dir: &str) {
    let status = Command::new("cargo")
        .args([
            "build",
            "--release",
            "--target",
            "wasm32-unknown-unknown",
            "--target-dir",
            target_dir,
            "--manifest-path",
            manifest,
        ])
        .env_remove("RUSTFLAGS")
        .env_remove("CARGO_ENCODED_RUSTFLAGS")
        .status()
        .expect("failed to build contract WASM");
    if !status.success() {
        panic!("contract WASM build failed for {manifest}");
    }
    let wasm_src = format!("{target_dir}/wasm32-unknown-unknown/release/{wasm_name}.wasm");
    std::fs::copy(&wasm_src, out).expect("failed to copy contract WASM");
}

fn main() {
    let contracts = [("lobby", "lobby_contract")];
    for (dir, wasm_name) in contracts {
        println!("cargo:rerun-if-changed=contract/{dir}/src/lib.rs");
        println!("cargo:rerun-if-changed=contract/{dir}/Cargo.toml");
        println!("cargo:rerun-if-changed=contract/{dir}/Cargo.lock");
        println!("cargo:rerun-if-changed=contract/{dir}/{wasm_name}.wasm");
    }

    let mut pending = Vec::new();
    for (dir, wasm_name) in contracts {
        let wasm = format!("contract/{dir}/{wasm_name}.wasm");
        let src = vec![
            format!("contract/{dir}/src/lib.rs"),
            format!("contract/{dir}/Cargo.toml"),
        ];
        if needs_build(&src, &wasm) {
            pending.push((dir, wasm_name, wasm));
        }
    }

    for (dir, wasm_name, wasm) in pending {
        let manifest = format!("contract/{dir}/Cargo.toml");
        let target_dir = format!("contract/{dir}/target");
        build_one(&manifest, wasm_name, &wasm, &target_dir);
    }
}
