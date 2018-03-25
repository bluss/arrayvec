
//!
//! This build script detects if we have new enough Rust
//!

extern crate version_check;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    if let Some((true, _)) = version_check::is_min_version("1.20.0") {
        println!("cargo:rustc-cfg=has_manuallydrop");
    }
}
