use crate::{util, wrapper, DEFAULT_CMD};
use anyhow::Result;
use std::{
    env::{var, var_os},
    ffi::OsStr,
    fs::copy,
    path::{Path, PathBuf},
    process::Command,
};

pub fn link(args: &[String]) -> Result<()> {
    let linker = linker()?;

    let mut command = Command::new(&linker);
    command.args(&args[1..]);
    util::exec_forwarding_output(command, true)?;

    // smoelius: Don't wrap if `RUSTC_WRAPPER` or `RUSTC_WORKSPACE_WRAPPER` is set. That usually
    // means that Clippy or Dylint is being run.
    if var_os("RUSTC_WRAPPER").is_none() && var_os("RUSTC_WORKSPACE_WRAPPER").is_none() {
        if let Some(path) = output_path(args.iter()) {
            if is_build_script(&path) {
                wrap(&linker, &path)?;
            }
        }
    }

    Ok(())
}

fn linker() -> Result<String> {
    if var_os("BUILD_WRAP_LD").is_some() {
        var("BUILD_WRAP_LD").map_err(Into::into)
    } else {
        Ok(String::from(util::DEFAULT_LD))
    }
}

fn output_path<'a, I>(mut iter: I) -> Option<PathBuf>
where
    I: Iterator<Item = &'a String>,
{
    while let Some(arg) = iter.next() {
        if arg == "-o" {
            if let Some(path) = iter.next() {
                return Some(path.into());
            }
        }
    }

    None
}

fn is_build_script(path: &Path) -> bool {
    path.file_name()
        .and_then(OsStr::to_str)
        .map_or(false, |name| name.starts_with("build_script_"))
}

fn wrap(linker: &str, build_script_path: &Path) -> Result<()> {
    let wrapper_package = wrapper::package(build_script_path)?;

    let mut command = util::cargo_build();
    if var_os("BUILD_WRAP_CMD").is_none() {
        command.env("BUILD_WRAP_CMD", DEFAULT_CMD);
    }
    // smoelius: When building the wrapper, do *not* use `build-wrap`.
    command.args([
        "--config",
        &format!("target.'cfg(all())'.linker = '{linker}'"),
    ]);
    // smoelius: `cd` into `wrapper_package`'s directory to avoid any `.cargo/config.toml` that may
    // be in ancestors of the current directory.
    command.current_dir(&wrapper_package);
    util::exec_forwarding_output(command, true)?;

    copy(
        wrapper_package
            .path()
            .join("target/debug/build_script_wrapper"),
        build_script_path,
    )?;

    Ok(())
}
