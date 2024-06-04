//! This file is included verbatim in the wrapper build script's src/main.rs file.

use anyhow::{anyhow, bail, ensure, Context, Result};
use once_cell::sync::Lazy;
use std::{
    env,
    fs::{canonicalize, set_permissions, Permissions},
    io::Write,
    os::unix::{ffi::OsStrExt, fs::PermissionsExt},
    path::Path,
    process::{Command, Output, Stdio},
    str::Utf8Error,
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
/// If `command` cannot be executed, or if `failure_is_error` is true and `command` failed.
pub fn exec_forwarding_output(mut command: Command, failure_is_error: bool) -> Result<Output> {
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());

    let output = command.output()?;

    // smoelius: Stdout *must* be forwarded.
    // See: https://doc.rust-lang.org/cargo/reference/build-scripts.html#life-cycle-of-a-build-script
    // `print!` and `eprint!` are used so that `libtest` will capture them.
    print!("{}", String::from_utf8_lossy(&output.stdout));
    eprint!("{}", String::from_utf8_lossy(&output.stderr));

    std::io::stdout().flush()?;
    std::io::stderr().flush()?;

    if !output.status.success() {
        if failure_is_error {
            bail!("command failed: {command:?}");
        }
        eprintln!("command failed: {command:?}");
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
    let expanded_args = split_and_expand(&temp_path)?;

    let allow_enabled = enabled("BUILD_WRAP_ALLOW");

    let mut command = Command::new(&expanded_args[0]);
    command.args(&expanded_args[1..]);
    let output = exec_forwarding_output(command, !allow_enabled)?;

    // smoelius: We should arrive at this `if` with `!output.status.success()` only when
    // `BUILD_WRAP_ALLOW` is enabled.
    if !output.status.success() {
        debug_assert!(allow_enabled);
        let command = Command::new(&temp_path);
        let _: Output = exec_forwarding_output(command, true)?;
    }

    drop(temp_path);

    Ok(())
}

pub trait ToUtf8 {
    fn to_utf8(&self) -> std::result::Result<&str, Utf8Error>;
}

impl<T: AsRef<Path>> ToUtf8 for T {
    fn to_utf8(&self) -> std::result::Result<&str, Utf8Error> {
        std::str::from_utf8(self.as_ref().as_os_str().as_bytes())
    }
}

pub fn split_and_expand(build_script_path: &Path) -> Result<Vec<String>> {
    let cmd =
        option_env!("BUILD_WRAP_CMD").ok_or_else(|| anyhow!("`BUILD_WRAP_CMD` is undefined"))?;
    let args = split_escaped(cmd)?;
    let expanded_args = args
        .into_iter()
        .map(|arg| expand_cmd(&arg, build_script_path))
        .collect::<Result<Vec<_>>>()?;
    eprintln!("expanded `BUILD_WRAP_CMD`: {:#?}", &expanded_args);
    ensure!(
        !expanded_args.is_empty(),
        "expanded `BUILD_WRAP_CMD` is empty or all whitespace"
    );

    Ok(expanded_args)
}

fn split_escaped(mut s: &str) -> Result<Vec<String>> {
    let mut v = vec![String::new()];

    while let Some(i) = s.find(|c: char| c.is_ascii_whitespace() || c == '\\') {
        debug_assert!(!v.is_empty());
        // smoelius: Only the last string in `v` can be empty.
        debug_assert!(v
            .iter()
            .position(String::is_empty)
            .map_or(true, |i| i == v.len() - 1));

        let c = s.as_bytes()[i];

        v.last_mut().unwrap().push_str(&s[..i]);

        s = &s[i + 1..];

        // smoelius: `i` shouldn't be needed anymore.
        #[allow(unused_variables, clippy::let_unit_value)]
        let i = ();

        if c.is_ascii_whitespace() {
            if !v.last().unwrap().is_empty() {
                v.push(String::new());
            }
            continue;
        }

        // smoelius: If the previous `if` fails, then `c` must be a backslash.
        if !s.is_empty() {
            let c = s.as_bytes()[0];
            // smoelius: Verify that `c` is a legally escapable character before subslicing `s`.
            ensure!(
                c.is_ascii_whitespace() || c == b'\\',
                "illegally escaped character"
            );
            s = &s[1..];
            v.last_mut().unwrap().push(c as char);
            continue;
        }

        bail!("trailing backslash");
    }

    // smoelius: Push whatever is left.
    v.last_mut().unwrap().push_str(s);

    if v.last().unwrap().is_empty() {
        v.pop();
    }

    debug_assert!(!v.iter().any(String::is_empty));

    Ok(v)
}

fn expand_cmd(mut cmd: &str, build_script_path: &Path) -> Result<String> {
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

static PRIVATE_TMPDIR: Lazy<Option<String>> = Lazy::new(|| {
    var("TMPDIR").ok().and_then(|value| {
        let path = canonicalize(value).ok()?;
        if path.starts_with("/private") {
            path.to_utf8().map(ToOwned::to_owned).ok()
        } else {
            None
        }
    })
});

fn var(key: &str) -> Result<String, env::VarError> {
    if key == "PRIVATE_TMPDIR" {
        return PRIVATE_TMPDIR.clone().ok_or(env::VarError::NotPresent);
    }

    env::var(key)
}

fn enabled(name: &str) -> bool {
    env::var(name).map_or(false, |value| value != "0")
}

#[cfg(test)]
mod test {
    use anyhow::Result;
    use std::{env::set_var, path::Path};

    #[test]
    fn expand_cmd() {
        set_var("KEY", "VALUE");

        let successes = [
            ("left path right", "{}"),
            ("left VALUE right", "{KEY}"),
            ("left { right", "{{"),
            ("left } right", "}}"),
        ];

        let failures = [
            ("environment variable `UNKNOWN` not found", "{UNKNOWN}"),
            ("unbalanced '{'", "{"),
            ("unbalanced '}'", "}"),
        ];

        for (expected, s) in successes {
            assert_eq!(expected, surround_and_expand(s).unwrap());
        }

        for (expected, s) in failures {
            assert_eq!(expected, surround_and_expand(s).unwrap_err().to_string());
        }
    }

    fn surround_and_expand(s: &str) -> Result<String> {
        let cmd = String::from("left ") + s + " right";
        super::expand_cmd(&cmd, Path::new("path"))
    }
}
