//! This file is included verbatim in the wrapper build script's src/main.rs file.

use anyhow::{anyhow, ensure, Result};
use std::{
    fs::{set_permissions, Permissions},
    io::Write,
    os::unix::fs::PermissionsExt,
    process::{Command, Output, Stdio},
};
use tempfile::NamedTempFile;

/// Executes `command`, forwards its output to stdout and stderr, and optionally checks whether
/// `command` succeeded.
///
/// Called by [`unpack_and_exec`]. Since this file is included in the wrapper build script's
/// src/main.rs file, `exec` should appear here, alongside [`unpack_and_exec`].
///
/// # Errors
///
/// If `command` cannot be executed, or if `require_success` is true and `command` failed.
pub fn exec(mut command: Command, require_success: bool) -> Result<Output> {
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());

    let output = command.output()?;

    // smoelius: Stdout *must* be forwarded.
    // See: https://doc.rust-lang.org/cargo/reference/build-scripts.html#life-cycle-of-a-build-script
    // `println!` and `eprintln!` are used so that `libtest` will capture them.
    println!("{}", String::from_utf8_lossy(&output.stdout));
    eprintln!("{}", String::from_utf8_lossy(&output.stderr));

    if require_success {
        ensure!(output.status.success(), "command failed: {command:?}");
    }

    Ok(output)
}

/// Essentially the body of the wrapper build script's `main` function. Not called by `build-wrap`
/// itself.
#[allow(dead_code)]
fn unpack_and_exec(bytes: &[u8]) -> Result<()> {
    let (mut file, temp_path) = NamedTempFile::new().map(NamedTempFile::into_parts)?;

    file.write_all(bytes)?;

    drop(file);

    set_permissions(&temp_path, Permissions::from_mode(0o755))?;

    let s =
        option_env!("BUILD_WRAP_CMD").ok_or_else(|| anyhow!("`BUILD_WRAP_CMD` is undefined"))?;
    let args = s.split_ascii_whitespace().collect::<Vec<_>>();
    ensure!(!args.is_empty(), "`BUILD_WRAP_CMD` is empty");

    let mut command = Command::new(args[0]);
    command.args(&args[1..]);
    command.arg(&temp_path);
    let _: Output = exec(command, true)?;

    drop(temp_path);

    Ok(())
}
