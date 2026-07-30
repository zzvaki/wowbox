mod build_env;

use std::{env, fs, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-env-changed=CURSEFORGE_API_KEY");
    let env_local =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("Cargo manifest directory"))
            .join("../.env.local");
    println!("cargo:rerun-if-changed={}", env_local.display());

    let api_key = env::var("CURSEFORGE_API_KEY")
        .ok()
        .and_then(|value| build_env::normalize(&value))
        .or_else(|| {
            fs::read_to_string(&env_local)
                .ok()
                .and_then(|contents| build_env::value(&contents, "CURSEFORGE_API_KEY"))
        });
    if let Some(api_key) = api_key {
        println!("cargo:rustc-env=CURSEFORGE_API_KEY={api_key}");
    }

    tauri_build::build()
}
