use anyhow::{bail, Context, Result};
use once_cell::sync::Lazy;
use std::{
    env::{args, current_exe},
    fs::read_to_string,
    io::{stdout, IsTerminal},
};

mod linking;
mod util;
mod wrapper;

const LINUX_DEFAULT_CMD: &str = "bwrap
    --ro-bind / /
    --dev-bind /dev /dev
    --bind {OUT_DIR} {OUT_DIR}
    --bind /tmp /tmp
    --unshare-net
    {}";

// smoelius: The following blog post is a useful `sandbox-exec` reference:
// https://7402.org/blog/2020/macos-sandboxing-of-folder.html
const MACOS_DEFAULT_CMD: &str = r#"sandbox-exec -p
(version\ 1)\
(deny\ default)\
(allow\ file-read*)\
(allow\ file-write*\ (subpath\ "/dev"))\
(allow\ file-write*\ (subpath\ "{OUT_DIR}"))\
(allow\ file-write*\ (subpath\ "{TMPDIR}"))\
(allow\ file-write*\ (subpath\ "{PRIVATE_TMPDIR}"))\
(allow\ process-exec)\
(allow\ process-fork)\
(allow\ sysctl-read)\
(deny\ network*)
{}"#;

const DEFAULT_CMD: &str = if cfg!(target_os = "linux") {
    LINUX_DEFAULT_CMD
} else {
    MACOS_DEFAULT_CMD
};

fn main() -> Result<()> {
    let args: Vec<String> = args().collect();

    run(&args)
}

fn run(args: &[String]) -> Result<()> {
    if args[1..]
        .iter()
        .all(|arg| matches!(arg.as_str(), "-h" | "--help"))
    {
        help();
        return Ok(());
    }

    linking::link(args)
}

static ENABLED: Lazy<&str> = Lazy::new(|| {
    if stdout().is_terminal() {
        "\x1b[1;32mENABLED\x1b[0m"
    } else {
        "ENABLED"
    }
});

static DISABLED: Lazy<&str> = Lazy::new(|| {
    if stdout().is_terminal() {
        "\x1b[1;31mDISABLED\x1b[0m"
    } else {
        "DISABLED"
    }
});

fn help() {
    println!(
        "{} {}

A linker replacement to help protect against malicious build scripts",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
    );
    let result = enabled();
    if matches!(result, Ok(true)) {
        println!(
            "
build-wrap is {}",
            *ENABLED
        );
        return;
    }
    let msg = result
        .err()
        .map(|error| format!(": {error}"))
        .unwrap_or_default();
    println!(
        r#"
build-wrap is {}{msg}

To enable build-wrap, create a `.cargo/config.toml` file in your home directory with the following contents:

```
[target.'cfg(all())']
linker = "build-wrap"
```"#,
        *DISABLED
    );
}

fn enabled() -> Result<bool> {
    let current_exe = current_exe()?;
    let Some(home) = home::home_dir() else {
        bail!("failed to determine home directory");
    };
    let path_buf = home.join(".cargo/config.toml");
    let contents = read_to_string(&path_buf)
        .with_context(|| format!("failed to read `{}`", path_buf.display()))?;
    let table = contents.parse::<toml::Table>()?;
    let Some(linker) = table
        .get("target")
        .and_then(toml::Value::as_table)
        .and_then(|table| table.get("cfg(all())"))
        .and_then(toml::Value::as_table)
        .and_then(|table| table.get("linker"))
        .and_then(toml::Value::as_str)
    else {
        bail!("`config.toml` has unexpected contents");
    };
    let path = util::which(linker)?;
    Ok(current_exe == path)
}

#[cfg(test)]
mod test {
    use regex::Regex;

    #[test]
    fn help() {
        super::run(&["build-wrap".to_owned(), "--help".to_owned()]).unwrap();
    }

    #[test]
    fn version() {
        super::run(&["build-wrap".to_owned(), "--version".to_owned()]).unwrap();
    }

    #[test]
    fn readme_contains_linux_default_cmd_with_comments() {
        super::util::assert_readme_contains_code_block(
            super::LINUX_DEFAULT_CMD.lines().map(str::trim_start),
            Some("sh"),
        );
    }

    #[test]
    fn readme_contains_linux_default_cmd_on_one_line() {
        let re = Regex::new("\\s+").unwrap();
        let cmd = re.replace_all(super::LINUX_DEFAULT_CMD, " ");
        super::util::assert_readme_contains_code_block(std::iter::once(cmd), Some("sh"));
    }
}
