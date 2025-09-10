use anyhow::{Context, Result};
use serde::Deserialize;
use std::{
    collections::HashMap,
    env::{consts, var_os},
    fs::{read_dir, read_to_string},
    path::Path,
    sync::LazyLock,
};

#[derive(Debug, Deserialize)]
pub struct Config {
    /// The operating system for which the stderr files are intended
    target_os: String,

    /// Optional name of the command used in `BUILD_WRAP_CMD`. If unspecified, a default is
    /// determined from `target_os`.
    name: Option<String>,

    /// Value of `BUILD_WRAP_CMD`
    build_wrap_cmd: Option<String>,
}

static COMMAND_NAMES: LazyLock<HashMap<&str, &str>> = LazyLock::new(|| {
    [("linux", "bwrap"), ("macos", "sandbox-exec")]
        .into_iter()
        .collect()
});

pub fn for_each_test_case(
    dir: impl AsRef<Path>,
    f: impl Fn(&str, Option<&str>, &Path, &str) -> Result<()>,
) -> Result<()> {
    let mut dirs = Vec::new();
    let mut test_cases = Vec::new();

    for result in read_dir(dir)? {
        let entry = result?;
        let path = entry.path();
        if path.is_dir() {
            dirs.push(path);
        } else {
            test_cases.push(path);
        }
    }

    dirs.sort();
    test_cases.sort();

    for dir in dirs {
        let config_toml_path = dir.join("config.toml");
        let contents = read_to_string(&config_toml_path)
            .with_context(|| format!("failed to read `{}`", config_toml_path.display()))?;
        let config = toml::from_str::<Config>(&contents)?;
        if config.target_os != consts::OS {
            continue;
        }
        let name = config
            .name
            .as_deref()
            .or_else(|| COMMAND_NAMES.get(config.target_os.as_str()).copied())
            .unwrap();
        for test_case in &test_cases {
            let file_stem = test_case
                .file_stem()
                .unwrap_or_else(|| panic!("`{}` has no file stem", test_case.display()));
            if let Some(testname) = var_os("TESTNAME") {
                if file_stem != testname {
                    return Ok(());
                }
            }
            let stderr_path = dir.join(file_stem).with_extension("stderr");
            let stderr = read_to_string(&stderr_path)
                .with_context(|| format!("failed to read `{}`", stderr_path.display()))?;
            f(name, config.build_wrap_cmd.as_deref(), test_case, &stderr)?;
        }
    }

    Ok(())
}
