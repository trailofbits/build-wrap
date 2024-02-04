//! This file must remain in the tests folder to ensure that `CARGO_BIN_EXE_build-wrap` is defined
//! at compile time.
//! See: https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-crates

use anyhow::Result;
use std::{
    fs::{copy, create_dir, write},
    path::Path,
    process::Command,
};
use tempfile::{tempdir, TempDir};

#[path = "../src/util/mod.rs"]
mod main_util;
pub use main_util::*;

#[must_use]
pub fn build_with_build_wrap() -> Command {
    let build_wrap = env!("CARGO_BIN_EXE_build-wrap");

    let mut command = main_util::cargo_build();
    command.args([
        "--config",
        &format!("target.'cfg(all())'.linker = '{build_wrap}'"),
    ]);

    command
}

pub fn temp_package(build_script_path: &Path) -> Result<TempDir> {
    let tempdir = tempdir()?;

    write(tempdir.path().join("Cargo.toml"), CARGO_TOML)?;
    copy(build_script_path, tempdir.path().join("build.rs"))?;
    create_dir(tempdir.path().join("src"))?;
    write(tempdir.path().join("src/lib.rs"), "")?;

    Ok(tempdir)
}

const CARGO_TOML: &str = r#"
[package]
name = "temp-package"
version = "0.1.0"
edition = "2021"
publish = false
"#;
