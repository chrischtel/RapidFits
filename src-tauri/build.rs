use std::fs;
use std::path::PathBuf;

fn main() {
    // Copy cfitsio dependencies to build output directory
    let vcpkg_bin = PathBuf::from("vcpkg_installed/x64-windows/bin");
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let out_path = PathBuf::from(&out_dir);
    let target_dir = out_path
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .expect("Failed to find target directory")
        .to_path_buf();

    println!("cargo:rerun-if-changed=vcpkg_installed/x64-windows/bin");

    // Copy DLLs
    let dlls = ["cfitsio.dll", "zlib1.dll"];
    for dll in &dlls {
        let src = vcpkg_bin.join(dll);
        let dst = target_dir.join(dll);

        if src.exists() {
            if let Err(e) = fs::copy(&src, &dst) {
                eprintln!("Failed to copy {}: {}", dll, e);
            } else {
                println!("Copied {} to {:?}", dll, dst);
            }
        } else {
            eprintln!("{} not found at {:?}", dll, src);
        }
    }

    tauri_build::build()
}
