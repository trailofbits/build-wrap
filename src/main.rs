use anyhow::{bail, Context, Result};
use once_cell::sync::Lazy;
use regex::Regex;
use std::{
    collections::BTreeMap,
    env::{args, current_exe},
    fs::read_to_string,
    io::{stdout, IsTerminal},
    path::Path,
    str::FromStr,
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
const MACOS_DEFAULT_CMD: &str = "sandbox-exec -f {BUILD_WRAP_PROFILE_PATH} {}";

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

A linker replacement to help protect against malicious build scripts
",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
    );
    let result = enabled();
    if matches!(result, Ok(true)) {
        let enabled = *ENABLED;
        println!("build-wrap is {enabled}");
        return;
    }
    let disabled = *DISABLED;
    let msg = result
        .err()
        .map(|error| format!(": {error}"))
        .unwrap_or_default();
    println!(
        r#"build-wrap is {disabled}{msg}

To enable build-wrap, create a `.cargo/config.toml` file in your home directory with the following contents:

```
[target.'cfg(all())']
linker = "build-wrap"
```{}"#,
        if noble_numbat_or_later().unwrap_or(cfg!(target_os = "linux")) {
            "

And install the Bubblewrap AppArmor profile with the following commands:

```
sudo apt install apparmor-profiles
sudo cp /usr/share/apparmor/extra-profiles/bwrap-userns-restrict /etc/apparmor.d
sudo systemctl reload apparmor
```"
        } else {
            ""
        }
    );
}

static BWRAP_APPARMOR_PROFILE_PATH: Lazy<&Path> =
    Lazy::new(|| Path::new("/etc/apparmor.d/bwrap-userns-restrict"));

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
    if current_exe != path {
        return Ok(false);
    }
    if noble_numbat_or_later()? && !BWRAP_APPARMOR_PROFILE_PATH.try_exists()? {
        bail!("`{}` does not exist", BWRAP_APPARMOR_PROFILE_PATH.display());
    }
    Ok(true)
}

static OS_RELEASE_PATH: Lazy<&Path> = Lazy::new(|| Path::new("/etc/os-release"));

static VERSION_ID_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"([0-9]+)\.[0-9]+").unwrap());

fn noble_numbat_or_later() -> Result<bool> {
    if !OS_RELEASE_PATH.try_exists()? {
        return Ok(false);
    }
    let map = parse_env_file(&OS_RELEASE_PATH)?;
    let Some(version_id) = map.get("VERSION_ID") else {
        bail!(
            "`{}` does not contain `VERSION_ID`",
            OS_RELEASE_PATH.display()
        );
    };
    let Some(captures) = VERSION_ID_RE.captures(version_id) else {
        bail!("failed to parse version id: {:?}", version_id);
    };
    assert_eq!(2, captures.len());
    let version_major = u64::from_str(captures.get(1).unwrap().as_str())?;
    Ok(version_major >= 24)
}

static ENV_LINE_RE: Lazy<Regex> = Lazy::new(|| Regex::new("([A-Za-z0-9_]+)=(.*)").unwrap());

fn parse_env_file(path: &Path) -> Result<BTreeMap<String, String>> {
    let mut map = BTreeMap::new();
    let contents = read_to_string(path)?;
    for line in contents.lines() {
        let Some(captures) = ENV_LINE_RE.captures(line) else {
            bail!("failed to parse line: {:?}", line);
        };
        assert_eq!(3, captures.len());
        let key = captures.get(1).unwrap().as_str();
        let mut value = captures.get(2).unwrap().as_str();
        if value.len() >= 2 && value.starts_with('"') && value.ends_with('"') {
            value = &value[1..value.len() - 1];
        }
        map.insert(key.to_owned(), value.to_owned());
    }
    Ok(map)
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

    #[test]
    fn readme_contains_macos_default_cmd() {
        super::util::assert_readme_contains_code_block(
            std::iter::once(super::MACOS_DEFAULT_CMD),
            Some("sh"),
        );
    }
}
