//! This file is included verbatim in the wrapper build script's src/main.rs file.

use anyhow::{anyhow, bail, ensure, Context, Result};
use std::{
    env::var,
    fs::{set_permissions, Permissions},
    io::Write,
    os::unix::fs::PermissionsExt,
    path::Path,
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

    // smoelius: The `BUILD_WRAP_CMD` used is the one set when set when the wrapper build script is
    // compiled, not when it is run. So if the wrapped build script prints the following and the
    // environment variable changes, those facts alone will not cause the wrapper build script
    // to be rebuilt:
    // ```
    // cargo:rerun-if-env-changed=BUILD_WRAP_CMD
    // ```
    // They will cause the wrapped build script to be rerun, however.
    let cmd =
        option_env!("BUILD_WRAP_CMD").ok_or_else(|| anyhow!("`BUILD_WRAP_CMD` is undefined"))?;
    let expanded_cmd = __expand_cmd(cmd, &temp_path)?;
    let expanded_args = expanded_cmd.split_ascii_whitespace().collect::<Vec<_>>();
    eprintln!("expanded `BUILD_WRAP_CMD`: {:#?}", &expanded_args);
    ensure!(
        !expanded_args.is_empty(),
        "expanded `BUILD_WRAP_CMD` is empty or all whitespace"
    );

    let mut command = Command::new(expanded_args[0]);
    command.args(&expanded_args[1..]);
    let _: Output = exec(command, true)?;

    drop(temp_path);

    Ok(())
}

pub trait ToUtf8 {
    // smoelius: `anyhow::Result` in a trait is kind of ick.
    fn to_utf8(&self) -> Result<&str>;
}

impl<T: AsRef<Path>> ToUtf8 for T {
    fn to_utf8(&self) -> Result<&str> {
        self.as_ref()
            .to_str()
            .ok_or_else(|| anyhow!("not valid utf-8"))
    }
}

// smoelius: `__expand_cmd` is `pub` simply to allow testing it in an integration test. It is not
// meant to be used outside of this module.
pub fn __expand_cmd(mut cmd: &str, build_script_path: &Path) -> Result<String> {
    let build_script_path_as_str = build_script_path.to_utf8()?;

    let mut buf = String::new();

    while let Some(i) = cmd.find(['{', '}']) {
        let c = cmd.as_bytes()[i];

        buf.push_str(&cmd[..i]);

        cmd = &cmd[i + 1..];

        // smoelius: `i` shouldn't be needed anymore.
        #[allow(unused_variables, clippy::let_unit_value)]
        let i = ();

        // smoelius: Escaped `{` or `}`?
        if !cmd.is_empty() && cmd.as_bytes()[0] == c {
            buf.push(c as char);
            cmd = &cmd[1..];
            continue;
        }

        if c == b'{' {
            if let Some(j) = cmd.find('}') {
                if j == 0 {
                    buf.push_str(build_script_path_as_str);
                } else {
                    let key = &cmd[..j];
                    let value = var(key)
                        .with_context(|| format!("environment variable `{key}` not found"))?;
                    buf.push_str(&value);
                }
                cmd = &cmd[j + 1..];
                continue;
            }
        }

        bail!("unbalanced '{}'", c as char);
    }

    // smoelius: Push whatever is left.
    buf.push_str(cmd);

    Ok(buf)
}
