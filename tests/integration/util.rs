//! This file must remain in the tests folder to ensure that `CARGO_BIN_EXE_build-wrap` is defined
//! at compile time. See [Environment variables Cargo sets for crates].
//!
//! [Environment variables Cargo sets for crates]: https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-crates

// smoelius: Use this module with `pub` to avoid "unused ..." warnings.
// See: https://users.rust-lang.org/t/invalid-dead-code-warning-for-submodule-in-integration-test/80259/2

use anyhow::{bail, Result};
use cargo_metadata::{Metadata, MetadataCommand};
use snapbox::assert_data_eq;
use std::{
    env,
    ffi::{OsStr, OsString},
    fs::{copy, create_dir, write, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process::Command,
    sync::LazyLock,
};

#[path = "../../src/util/mod.rs"]
mod main_util;
pub use main_util::*;

#[ctor::ctor]
fn initialize() {
    env::set_var("CARGO_TERM_COLOR", "never");
}

#[must_use]
pub fn build_with_build_wrap() -> Command {
    let build_wrap = env!("CARGO_BIN_EXE_build-wrap");

    let mut command = cargo_build();
    command.args([
        "--config",
        &format!("target.'cfg(all())'.linker = '{build_wrap}'"),
    ]);

    command
}

#[must_use]
pub fn build_with_default_linker() -> Command {
    let mut command = cargo_build();
    command.args([
        "--config",
        &format!("target.'cfg(all())'.linker = '{DEFAULT_LD}'"),
    ]);

    command
}

pub fn temp_package<'a, 'b>(
    build_script_path: Option<impl AsRef<Path>>,
    dependencies: impl IntoIterator<Item = (&'a str, &'b str)>,
) -> Result<tempfile::TempDir> {
    let tempdir = tempdir()?;

    write(tempdir.path().join("Cargo.toml"), CARGO_TOML)?;
    if let Some(build_script_path) = build_script_path {
        copy(build_script_path, tempdir.path().join("build.rs"))?;
    }
    create_dir(tempdir.path().join("src"))?;
    write(tempdir.path().join("src/lib.rs"), "")?;

    let mut iter = dependencies.into_iter().peekable();

    if iter.peek().is_some() {
        let mut file = OpenOptions::new()
            .append(true)
            .open(tempdir.path().join("Cargo.toml"))?;
        writeln!(file, "\n[dependencies]")?;
        for (name, version) in iter {
            writeln!(file, r#"{name} = "={version}""#)?;
        }
    }

    Ok(tempdir)
}

const CARGO_TOML: &str = r#"
[package]
name = "temp-package"
version = "0.1.0"
edition = "2021"
publish = false

[build-dependencies]
libc = { version = "0.2", optional = true }
rustc_version = { version = "0.4", optional = true }
"#;

static METADATA: LazyLock<Metadata> =
    LazyLock::new(|| MetadataCommand::new().no_deps().exec().unwrap());

/// Creates a temporary directory in `build-wrap`'s target directory.
///
/// Useful if you want to verify that writing outside of the temporary directory is forbidden, but
/// `/tmp` is writeable, for example.
pub fn tempdir() -> Result<tempfile::TempDir> {
    tempfile::tempdir_in(&METADATA.target_directory).map_err(Into::into)
}

#[derive(Debug)]
pub enum TestCase<'a> {
    BuildScript(&'a Path),
    ThirdParty(&'a str, &'a str),
}

pub fn test_case(build_wrap_cmd: Option<&str>, test_case: &TestCase, stderr_expected: &str) {
    let temp_package = match *test_case {
        TestCase::BuildScript(path) => temp_package(Some(path), []),
        TestCase::ThirdParty(name, version) => temp_package(None::<&Path>, [(name, version)]),
    }
    .unwrap();

    let mut command = build_with_build_wrap();
    // smoelius: `--all-features` to enable optional build dependencies.
    command.arg("--all-features");
    if let Some(build_wrap_cmd) = build_wrap_cmd {
        prepend_scripts_to_path(&mut command).unwrap();
        prepend_out_dir_to_path(&mut command).unwrap();
        command.env("BUILD_WRAP_CMD", build_wrap_cmd);
    }
    command.current_dir(&temp_package);

    let output = exec_forwarding_output(command, false).unwrap();
    assert_eq!(
        stderr_expected.is_empty(),
        output.status.success(),
        "{test_case:?} failed in `{}`",
        temp_package.path().display()
    );

    if stderr_expected.is_empty() {
        return;
    }

    let stderr_actual = std::str::from_utf8(&output.stderr).unwrap();
    assert_data_eq!(stderr_actual, stderr_expected);
}

// smoelius: `prepend_scripts_to_path` allows `BUILD_WRAP_CMD`s to refer to files in the scripts
// directory.
fn prepend_scripts_to_path(command: &mut Command) -> Result<()> {
    let scripts = Path::new(env!("CARGO_MANIFEST_DIR")).join("scripts");
    prepend_to_path(command, scripts)?;
    Ok(())
}

// smoelius: `prepend_out_dir_to_path` allows `BUILD_WRAP_CMD`s to refer to files in `OUT_DIR`.
fn prepend_out_dir_to_path(command: &mut Command) -> Result<()> {
    let out_dir = PathBuf::from(env!("OUT_DIR"));
    prepend_to_path(command, out_dir)?;
    Ok(())
}

pub fn prepend_to_path(command: &mut Command, path: PathBuf) -> Result<()> {
    let paths = if let Some(paths) = get_env(command, "PATH") {
        prepend_path(path, paths)?
    } else if let Some(paths) = env::var_os("PATH") {
        prepend_path(path, &paths)?
    } else {
        bail!("`PATH` is unset");
    };
    command.env("PATH", paths);
    Ok(())
}

fn get_env(command: &Command, needle: impl AsRef<OsStr>) -> Option<&OsStr> {
    let needle = needle.as_ref();
    command
        .get_envs()
        .find_map(|(key, val)| if key == needle { val } else { None })
}

pub fn prepend_path(path: PathBuf, paths: &OsStr) -> Result<OsString> {
    let paths_split = env::split_paths(&paths);
    let paths_chained = std::iter::once(path).chain(paths_split);
    let paths_joined = env::join_paths(paths_chained)?;
    Ok(paths_joined)
}
