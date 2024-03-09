// smoelius: Copied from:
// https://github.com/djc/rustc-version-rs/blob/9cdb26683edb35a31573f5322594ef07e43aa142/README.md?plain=1#L37-L65

// This could be a cargo build script

use rustc_version::{version, version_meta, Channel, Version};

fn main() {
    // Assert we haven't travelled back in time
    assert!(version().unwrap().major >= 1);

    // Set cfg flags depending on release channel
    match version_meta().unwrap().channel {
        Channel::Stable => {
            println!("cargo:rustc-cfg=RUSTC_IS_STABLE");
        }
        Channel::Beta => {
            println!("cargo:rustc-cfg=RUSTC_IS_BETA");
        }
        Channel::Nightly => {
            println!("cargo:rustc-cfg=RUSTC_IS_NIGHTLY");
        }
        Channel::Dev => {
            println!("cargo:rustc-cfg=RUSTC_IS_DEV");
        }
    }

    // Check for a minimum version
    if version().unwrap() >= Version::parse("1.4.0").unwrap() {
        println!("cargo:rustc-cfg=compiler_has_important_bugfix");
    }
}
