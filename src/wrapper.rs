use crate::util::ToUtf8;
use anyhow::{anyhow, Result};
use std::{
    fs::{create_dir, rename, write},
    path::Path,
};
use tempfile::{tempdir, NamedTempFile, TempDir};

#[allow(clippy::disallowed_methods)]
pub fn package(build_script_path: &Path) -> Result<TempDir> {
    let parent = build_script_path
        .parent()
        .ok_or_else(|| anyhow!("failed to get `build_script_path` parent"))?;

    let temp_file = NamedTempFile::new_in(parent)?;

    let (_file, sibling_path) = temp_file.keep()?;

    rename(build_script_path, &sibling_path)?;

    let sibling_path_as_str = sibling_path.to_utf8()?;

    let tempdir = tempdir()?;

    write(tempdir.path().join("Cargo.toml"), CARGO_TOML)?;
    create_dir(tempdir.path().join("src"))?;
    write(
        tempdir.path().join("src/main.rs"),
        main_rs(sibling_path_as_str),
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
xdg = "2.5"
"#;

/// A wrapper build script's src/main.rs consists of the following:
///
/// - the contents of util/common.rs (included verbatim)
/// - the path of the renamed original build script (`PATH`)
/// - a `main` function
///
/// See [`package`].
fn main_rs(sibling_path_as_str: &str) -> Vec<u8> {
    [
        COMMON_RS,
        format!(
            r#"
const PATH: &str = "{sibling_path_as_str}";

fn main() -> Result<()> {{
    exec_sibling(PATH)
}}
"#,
        )
        .as_bytes(),
    ]
    .concat()
}

const COMMON_RS: &[u8] = include_bytes!("util/common.rs");
