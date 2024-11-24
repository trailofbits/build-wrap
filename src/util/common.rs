//! This file is included verbatim in the wrapper build script's src/main.rs file.

use anyhow::{anyhow, bail, ensure, Context, Result};
use once_cell::sync::Lazy;
use std::{
    env,
    fs::canonicalize,
    io::Write,
    os::unix::ffi::OsStrExt,
    path::Path,
    process::{Command, Output, Stdio},
    str::Utf8Error,
};

#[allow(dead_code)]
const DEFAULT_PROFILE: &str = r#"(version 1)
(deny default)
(allow file-read*)                               ;; Allow read-only access everywhere
(allow file-write* (subpath "/dev"))             ;; Allow write access to /dev
(allow file-write* (subpath "{OUT_DIR}"))        ;; Allow write access to `OUT_DIR`
(allow file-write* (subpath "{TMPDIR}"))         ;; Allow write access to `TMPDIR`
(allow file-write* (subpath "{PRIVATE_TMPDIR}")) ;; Allow write access to `PRIVATE_TMPDIR` (see below)
(allow process-exec)                             ;; Allow `exec`
(allow process-fork)                             ;; Allow `fork`
(allow sysctl-read)                              ;; Allow reading kernel state
(deny network*)                                  ;; Deny network access
"#;

/// Executes `command`, forwards its output to stdout and stderr, and optionally checks whether
/// `command` succeeded.
///
/// Called by [`exec_sibling`]. Since this file is included in the wrapper build script's
/// src/main.rs file, `exec_forwarding_output` should appear here, alongside [`exec_sibling`].
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
fn exec_sibling(sibling_path_as_str: &str) -> Result<()> {
    let current_exe = env::current_exe()?;

    let parent = current_exe
        .parent()
        .ok_or_else(|| anyhow!("failed to get `current_exe` parent"))?;

    let sibling_path = Path::new(sibling_path_as_str);

    assert!(sibling_path.starts_with(parent));

    // smoelius: The `BUILD_WRAP_CMD` used is the one set when set when the wrapper build script is
    // compiled, not when it is run. So if the wrapped build script prints the following and the
    // environment variable changes, those facts alone will not cause the wrapper build script
    // to be rebuilt:
    // ```
    // cargo:rerun-if-env-changed=BUILD_WRAP_CMD
    // ```
    // They will cause the wrapped build script to be rerun, however.
    let expanded_args = split_and_expand(sibling_path)?;

    let allow_enabled = enabled("BUILD_WRAP_ALLOW");

    let mut command = Command::new(&expanded_args[0]);
    command.args(&expanded_args[1..]);
    let output = exec_forwarding_output(command, !allow_enabled)?;

    // smoelius: We should arrive at this `if` with `!output.status.success()` only when
    // `BUILD_WRAP_ALLOW` is enabled.
    if !output.status.success() {
        debug_assert!(allow_enabled);
        let command = Command::new(sibling_path);
        let _: Output = exec_forwarding_output(command, true)?;
    }

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
        .map(|arg| expand(&arg, Some(build_script_path)))
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

fn expand(mut cmd: &str, build_script_path: Option<&Path>) -> Result<String> {
    let build_script_path_as_str = build_script_path
        .map(|path| path.to_utf8().map(ToOwned::to_owned))
        .transpose()?;

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
                    let s = build_script_path_as_str
                        .as_ref()
                        .ok_or_else(|| anyhow!("build script path is unavailable"))?;
                    buf.push_str(s);
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

#[cfg(target_os = "macos")]
static BUILD_WRAP_PROFILE_PATH: Lazy<String> = Lazy::new(|| {
    let tempfile = tempfile::NamedTempFile::new().unwrap();
    let (mut file, temp_path) = tempfile.into_parts();
    let profile = var("BUILD_WRAP_PROFILE").unwrap_or(DEFAULT_PROFILE.to_owned());
    let expanded_profile = expand(&profile, None).unwrap();
    file.write_all(expanded_profile.as_bytes()).unwrap();
    let path = temp_path.keep().unwrap();
    path.to_utf8().map(ToOwned::to_owned).unwrap()
});

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
    #[cfg(target_os = "macos")]
    if key == "BUILD_WRAP_PROFILE_PATH" {
        return Ok(BUILD_WRAP_PROFILE_PATH.clone());
    }

    if key == "PRIVATE_TMPDIR" {
        return PRIVATE_TMPDIR.clone().ok_or(env::VarError::NotPresent);
    }

    env::var(key)
}

fn enabled(name: &str) -> bool {
    env::var(name).is_ok_and(|value| value != "0")
}

#[cfg(test)]
pub use test::assert_readme_contains_code_block;

#[cfg(test)]
mod test {
    use anyhow::Result;
    use std::{env::set_var, fs::read_to_string, path::Path};

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
        super::expand(&cmd, Some(Path::new("path")))
    }

    #[test]
    fn readme_contains_default_profile() {
        assert_readme_contains_code_block(super::DEFAULT_PROFILE.lines(), None);
    }

    pub fn assert_readme_contains_code_block(
        lines: impl Iterator<Item = impl AsRef<str>>,
        language: Option<&str>,
    ) {
        let delimited_lines = std::iter::once(format!("```{}", language.unwrap_or_default()))
            .chain(lines.map(|s| s.as_ref().to_owned()))
            .chain(std::iter::once(String::from("```")))
            .collect::<Vec<_>>();
        let size = delimited_lines.len();
        let readme = read_to_string("README.md").unwrap();
        let readme_lines = readme
            .lines()
            .map(|line| {
                let index = line.find('#').unwrap_or(line.len());
                line[..index].trim()
            })
            .collect::<Vec<_>>();
        assert!(readme_lines.windows(size).any(|w| w == delimited_lines));
    }
}
