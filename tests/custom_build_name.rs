use anyhow::Result;
use std::{
    fs::{copy, create_dir, write},
    path::Path,
};
use tempfile::{tempdir, TempDir};

pub mod util;

#[test]
fn custom_build_name() {
    let temp_package = temp_package(Path::new("tests/cases/ping.rs")).unwrap();

    let mut command = util::build_with_build_wrap();
    command.current_dir(&temp_package);

    let output = util::exec(command, false).unwrap();
    assert!(!output.status.success());

    let stderr = std::str::from_utf8(&output.stderr).unwrap();
    assert!(
        stderr.contains("ping: socket: Operation not permitted"),
        "stderr does not contain expected string:\n```\n{stderr}\n```",
    );
}

#[allow(clippy::disallowed_methods)]
fn temp_package(build_script_path: &Path) -> Result<TempDir> {
    let tempdir = tempdir()?;

    write(tempdir.path().join("Cargo.toml"), CARGO_TOML)?;
    copy(
        build_script_path,
        tempdir.path().join("custom_build_name.rs"),
    )?;
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

build = "custom_build_name.rs"
"#;
