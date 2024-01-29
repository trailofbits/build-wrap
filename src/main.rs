use anyhow::Result;
use std::{
    env::{args, var, var_os},
    ffi::OsStr,
    fs::copy,
    path::{Path, PathBuf},
    process::Command,
};

mod util;
mod wrapper;

const DEFAULT_CMD: &str = "unshare --map-root-user --net";
const DEFAULT_LD: &str = "cc";

fn main() -> Result<()> {
    let linker = linker()?;

    let args: Vec<String> = args().collect();

    let mut command = Command::new(&linker);
    command.args(&args[1..]);
    util::exec(command, true)?;

    if let Some(path) = output_path(args.iter()) {
        if is_build_script(&path) {
            wrap(&linker, &path)?;
        }
    }

    Ok(())
}

fn linker() -> Result<String> {
    if var_os("BUILD_WRAP_LD").is_some() {
        var("BUILD_WRAP_LD").map_err(Into::into)
    } else {
        Ok(String::from(DEFAULT_LD))
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
        .map_or(false, |name| name.starts_with("build_script_build-"))
}

fn wrap(linker: &str, build_script_path: &Path) -> Result<()> {
    let wrapper_package = wrapper::package(build_script_path)?;

    let mut command = util::cargo_build();
    command.arg("--manifest-path");
    command.arg(wrapper_package.path().join("Cargo.toml"));
    if var_os("BUILD_WRAP_CMD").is_none() {
        command.env("BUILD_WRAP_CMD", DEFAULT_CMD);
    }
    // smoelius: When building the wrapper, do *not* use `build-wrap`.
    command.args([
        "--config",
        &format!("target.'cfg(all())'.linker = '{linker}'"),
    ]);
    util::exec(command, true)?;

    copy(
        wrapper_package
            .path()
            .join("target/debug/build_script_wrapper"),
        build_script_path,
    )?;

    Ok(())
}
