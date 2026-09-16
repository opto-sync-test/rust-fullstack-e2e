use std::{env, path::PathBuf};

fn main() {
    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("manifest dir"));
    let out = PathBuf::from(env::var("OUT_DIR").expect("out dir"));
    let outputs = ores_api_docs::write_page_build_outputs(&root, &out)
        .expect("generated page router build must succeed");
    for path in outputs.rerun_if_changed {
        println!("cargo:rerun-if-changed={}", path.display());
    }
}
