fn main() {
    println!("cargo:rerun-if-env-changed=CURSEFORGE_API_KEY");
    tauri_build::build()
}
