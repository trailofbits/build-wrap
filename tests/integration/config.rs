use anyhow::{Context, Result};
use serde::Deserialize;
use std::{
    collections::HashMap,
    env::{consts, var_os},
    fs::{read_dir, read_to_string},
    io::Write,
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
    let mut subdirs = Vec::new();
    let mut test_cases = Vec::new();

    for result in read_dir(dir)? {
        let entry = result?;
        let path = entry.path();
        if path.is_dir() {
            subdirs.push(path);
        } else {
            test_cases.push(path);
        }
    }

    subdirs.sort();
    test_cases.sort();

    // smoelius: Each `subdir` should contain:
    //
    // - a config.toml file encoding a `Config`
    // - a stderr file for each test case found in `dir`
    //
    // If a stderr file cannot be found for a test case in `dir`, a warning is issued but it does
    // not cause the test to fail.
    for subdir in subdirs {
        let config_toml_path = subdir.join("config.toml");
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
            if let Some(testname) = var_os("TESTNAME")
                && file_stem != testname
            {
                return Ok(());
            }
            let stderr_path = subdir.join(file_stem).with_extension("stderr");
            if !stderr_path.try_exists()? {
                #[allow(clippy::explicit_write)]
                writeln!(
                    std::io::stderr(),
                    "warning: `{}` does not exist",
                    stderr_path.display()
                )
                .unwrap();
                continue;
            }
            let stderr = read_to_string(&stderr_path)
                .with_context(|| format!("failed to read `{}`", stderr_path.display()))?;
            f(name, config.build_wrap_cmd.as_deref(), test_case, &stderr)?;
        }
    }

    Ok(())
}
