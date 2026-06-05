// build.rs
use std::path::PathBuf;

const LOUPE_VERSION: &str = "v0.3.1";
const LOUPE_REPO: &str = "georgemandis/loupe";

fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();

    if target_os == "linux" {
        println!("cargo:warning=loupe is not available on Linux, skipping");
        return;
    }

    // The loupe release archives contain a dylib on macOS and a DLL import lib on Windows.
    let (asset_name, lib_file, link_kind) = match (target_os.as_str(), target_arch.as_str()) {
        ("macos", "aarch64") => (
            format!("loupe-{}-macos-aarch64.tar.gz", LOUPE_VERSION),
            "libloupe.dylib",
            "dylib",
        ),
        ("macos", "x86_64") => (
            format!("loupe-{}-macos-x86_64.tar.gz", LOUPE_VERSION),
            "libloupe.dylib",
            "dylib",
        ),
        ("windows", "x86_64") => (
            format!("loupe-{}-windows-x86_64.zip", LOUPE_VERSION),
            "loupe.dll",
            "dylib",
        ),
        _ => {
            println!("cargo:warning=unsupported platform for loupe: {}-{}", target_os, target_arch);
            return;
        }
    };

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let lib_path = out_dir.join(lib_file);

    if lib_path.exists() {
        println!("cargo:rustc-link-search=native={}", out_dir.display());
        println!("cargo:rustc-link-lib={}=loupe", link_kind);
        link_frameworks(&target_os);
        return;
    }

    let url = format!(
        "https://github.com/{}/releases/download/{}/{}",
        LOUPE_REPO, LOUPE_VERSION, asset_name,
    );

    println!("cargo:warning=Downloading loupe from {}", url);

    let output = std::process::Command::new("curl")
        .args(["-fsSL", "-o"])
        .arg(out_dir.join(&asset_name).to_str().unwrap())
        .arg(&url)
        .output()
        .expect("failed to run curl");

    assert!(
        output.status.success(),
        "failed to download loupe: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let archive_path = out_dir.join(&asset_name);

    if asset_name.ends_with(".tar.gz") {
        let tar_gz = std::fs::File::open(&archive_path).unwrap();
        let tar = flate2::read::GzDecoder::new(tar_gz);
        let mut archive = tar::Archive::new(tar);
        archive.unpack(&out_dir).expect("failed to extract loupe archive");
    } else if asset_name.ends_with(".zip") {
        let status = std::process::Command::new("tar")
            .args(["-xf"])
            .arg(&archive_path)
            .arg("-C")
            .arg(&out_dir)
            .status()
            .expect("failed to extract zip");
        assert!(status.success(), "failed to extract loupe zip");
    }

    let extracted_lib = find_file_recursive(&out_dir, lib_file)
        .unwrap_or_else(|| panic!("{} not found in extracted archive", lib_file));

    if extracted_lib != lib_path {
        std::fs::copy(&extracted_lib, &lib_path).unwrap();
    }

    println!("cargo:rustc-link-search=native={}", out_dir.display());
    println!("cargo:rustc-link-lib={}=loupe", link_kind);
    link_frameworks(&target_os);
}

fn link_frameworks(target_os: &str) {
    if target_os == "macos" {
        for fw in &[
            "Vision", "CoreGraphics", "CoreImage", "ImageIO",
            "CoreVideo", "Foundation", "AppKit",
        ] {
            println!("cargo:rustc-link-lib=framework={}", fw);
        }
    }
}

fn find_file_recursive(dir: &PathBuf, name: &str) -> Option<PathBuf> {
    if dir.join(name).exists() {
        return Some(dir.join(name));
    }
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(found) = find_file_recursive(&path, name) {
                    return Some(found);
                }
            }
        }
    }
    None
}
