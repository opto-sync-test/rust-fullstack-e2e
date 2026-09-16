use std::{env, fs, path::PathBuf};

use sha2::{Digest, Sha256};

fn main() {
    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("manifest dir"));
    let out = PathBuf::from(env::var("OUT_DIR").expect("out dir"));
    let first_pass_dir = out.join("first-pass");
    let first = ores_api_docs::write_page_build_outputs(&root, &first_pass_dir)
        .expect("generated first-pass page build must succeed");
    for path in &first.rerun_if_changed {
        println!("cargo:rerun-if-changed={}", path.display());
    }

    let mut manifest = ores_api_docs::read_page_build_manifest(&first.manifest_path)
        .expect("first-pass manifest must parse");
    let final_assets = out.join("final-assets");
    fs::create_dir_all(&final_assets).expect("final asset dir");

    // `ores-stack finalize-assets` normally copies planned CSS to the final
    // content-addressed asset directory before native server compilation.
    for route in &manifest.routes {
        if let Some(css) = &route.css {
            fs::copy(
                first.asset_dir.join(&css.output_file),
                final_assets.join(&css.output_file),
            )
            .expect("copy planned css to final assets");
        }
    }

    let user = manifest
        .routes
        .iter_mut()
        .find(|route| route.canonical_path == "/users/{id}")
        .expect("hydrated user route");
    let wasm = user.wasm.as_mut().expect("user browser build plan");

    // Deterministic stand-ins for a real Leptos client builder. The important
    // integration boundary here is the finalized bytes -> hashes -> manifest ->
    // generated server route flow, which is the same second pass ores-stack uses.
    let js_bytes = b"export const oresHydratedUser = true;\n";
    let wasm_bytes = b"\0asm\x01\0\0\0ores-stack-test";
    let js_sha = sha256(js_bytes);
    let wasm_sha = sha256(wasm_bytes);
    let js_file = format!("page-{js_sha}.js");
    let wasm_file = format!("page-{wasm_sha}.wasm");
    fs::write(final_assets.join(&js_file), js_bytes).expect("write finalized js");
    fs::write(final_assets.join(&wasm_file), wasm_bytes).expect("write finalized wasm");

    wasm.js_sha256 = Some(js_sha.clone());
    wasm.js_output_file = Some(js_file.clone());
    wasm.js_public_path = Some(format!("/__ores/assets/{js_file}"));
    wasm.final_wasm_sha256 = Some(wasm_sha.clone());
    wasm.wasm_output_file = Some(wasm_file.clone());
    wasm.public_path = Some(format!("/__ores/assets/{wasm_file}"));

    let final_manifest = out.join("final/ores-page-manifest.json");
    fs::create_dir_all(final_manifest.parent().expect("final manifest parent"))
        .expect("final manifest dir");
    ores_api_docs::write_page_build_manifest(&final_manifest, &manifest)
        .expect("write finalized page manifest");

    let materialized = ores_api_docs::materialize_finalized_page_build(
        &root,
        &out,
        &final_manifest,
        &final_assets,
    )
    .expect("materialize finalized pages into cargo OUT_DIR");

    println!("cargo:rerun-if-changed={}", final_manifest.display());
    println!("cargo:rerun-if-changed={}", final_assets.display());
    println!("cargo:rustc-env=ORES_TEST_USER_WASM_SHA={wasm_sha}");
    println!(
        "cargo:rustc-env=ORES_TEST_USER_WASM_PATH=/__ores/assets/{wasm_file}"
    );
    println!("cargo:rustc-env=ORES_TEST_USER_JS_PATH=/__ores/assets/{js_file}");

    assert!(materialized.compile_glue_path == out.join("ores_pages.rs"));
    assert!(materialized.manifest_path == out.join("ores-page-manifest.json"));
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
