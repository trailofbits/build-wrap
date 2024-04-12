use crate::util::ToUtf8;
use anyhow::Result;
use std::{
    fs::{create_dir, write},
    path::Path,
};
use tempfile::{tempdir, TempDir};

#[allow(clippy::disallowed_methods)]
pub fn package(build_script_path: &Path) -> Result<TempDir> {
    let build_script_path_as_str = build_script_path.to_utf8()?;

    let tempdir = tempdir()?;

    write(tempdir.path().join("Cargo.toml"), CARGO_TOML)?;
    create_dir(tempdir.path().join("src"))?;
    write(
        tempdir.path().join("src/main.rs"),
        main_rs(build_script_path_as_str),
    )?;

    Ok(tempdir)
}

// smoelius: The dependencies listed here must be sufficient to compile util/common.rs.
const CARGO_TOML: &str = r#"
[package]
name = "build_script_wrapper"
version = "0.1.0"
edition = "2021"
publish = false

[dependencies]
anyhow = "1.0"
once_cell = "1.19"
tempfile = "3.10"
"#;

/// A wrapper build script's src/main.rs consists of the following:
///
/// - the contents of util/common.rs (included verbatim)
/// - the original build script as a byte slice (`BYTES`)
/// - a `main` function
///
/// See [`package`].
fn main_rs(build_script_path_as_str: &str) -> Vec<u8> {
    [
        COMMON_RS,
        format!(
            r#"
const BYTES: &[u8] = include_bytes!("{build_script_path_as_str}");

fn main() -> Result<()> {{
    unpack_and_exec(BYTES)
}}
"#,
        )
        .as_bytes(),
    ]
    .concat()
}

const COMMON_RS: &[u8] = include_bytes!("util/common.rs");
