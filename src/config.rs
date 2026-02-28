use std::{env, fs::read_to_string, path::Path, sync::LazyLock};

static CONFIG: LazyLock<Config> = LazyLock::new(Config::load);

#[derive(Default)]
struct Config {
    directories: Vec<String>,
    packages: Vec<String>,
}

impl Config {
    fn load() -> Self {
        let base_directories = xdg::BaseDirectories::new();
        let Some(path) = base_directories.find_config_file("build-wrap/config.toml") else {
            return Self::default();
        };
        Self::load_from(&path)
    }

    fn load_from(path: &Path) -> Self {
        let Ok(contents) = read_to_string(path) else {
            return Self::default();
        };
        let Ok(table) = contents.parse::<toml::Table>() else {
            return Self::default();
        };

        for key in table.keys() {
            if key != "allow" && key != "ignore" {
                eprintln!(
                    "warning: {}: unrecognized table `[{key}]`",
                    path.display(),
                );
            }
        }

        let mut directories = Vec::new();
        let mut packages = Vec::new();

        for section in ["allow", "ignore"] {
            if let Some(table) = table.get(section).and_then(toml::Value::as_table) {
                extend_from_string_array(&mut directories, table.get("directories"));
                extend_from_string_array(&mut packages, table.get("packages"));
            }
        }

        Self {
            directories,
            packages,
        }
    }
}

fn extend_from_string_array(vec: &mut Vec<String>, value: Option<&toml::Value>) {
    if let Some(array) = value.and_then(toml::Value::as_array) {
        for item in array {
            if let Some(s) = item.as_str() {
                vec.push(s.to_owned());
            }
        }
    }
}

pub fn allowed() -> bool {
    directory_allowed() || package_allowed()
}

pub fn directory_allowed() -> bool {
    let Ok(cwd) = env::current_dir() else {
        return false;
    };
    CONFIG.directories.iter().any(|d| cwd.starts_with(d))
}

fn package_allowed() -> bool {
    let Ok(name) = env::var("CARGO_PKG_NAME") else {
        return false;
    };
    CONFIG.packages.iter().any(|p| p == &name)
}

#[cfg(test)]
#[allow(clippy::disallowed_methods)]
mod test {
    use super::*;
    use std::fs::write;

    const EXAMPLE_CONFIG: &str = r#"
[allow]
directories = ["/home/user/project-a"]
packages = ["aws-lc-fips-sys"]

[ignore]
directories = ["/home/user/project-b"]
packages = ["svm-rs-builds"]
"#;

    #[test]
    fn parse_config() {
        let dir = tempfile::tempdir().unwrap();
        let path_buf = dir.path().join("config.toml");
        write(&path_buf, EXAMPLE_CONFIG).unwrap();

        let config = Config::load_from(&path_buf);

        assert_eq!(
            config.directories,
            vec!["/home/user/project-a", "/home/user/project-b"]
        );
        assert_eq!(config.packages, vec!["aws-lc-fips-sys", "svm-rs-builds"]);
    }

    #[test]
    fn missing_config() {
        let config = Config::load_from(Path::new("/nonexistent/config.toml"));
        assert!(config.directories.is_empty());
        assert!(config.packages.is_empty());
    }

    #[test]
    fn empty_config() {
        let dir = tempfile::tempdir().unwrap();
        let path_buf = dir.path().join("config.toml");
        write(&path_buf, "").unwrap();

        let config = Config::load_from(&path_buf);
        assert!(config.directories.is_empty());
        assert!(config.packages.is_empty());
    }

    #[test]
    fn allow_only_config() {
        let dir = tempfile::tempdir().unwrap();
        let path_buf = dir.path().join("config.toml");
        write(
            &path_buf,
            r#"
[allow]
packages = ["foo"]
"#,
        )
        .unwrap();

        let config = Config::load_from(&path_buf);
        assert!(config.directories.is_empty());
        assert_eq!(config.packages, vec!["foo"]);
    }

    #[test]
    fn ignore_only_config() {
        let dir = tempfile::tempdir().unwrap();
        let path_buf = dir.path().join("config.toml");
        write(
            &path_buf,
            r#"
[ignore]
directories = ["/tmp/myproject"]
"#,
        )
        .unwrap();

        let config = Config::load_from(&path_buf);
        assert_eq!(config.directories, vec!["/tmp/myproject"]);
        assert!(config.packages.is_empty());
    }

    #[test]
    fn unrecognized_table() {
        let dir = tempfile::tempdir().unwrap();
        let path_buf = dir.path().join("config.toml");
        write(
            &path_buf,
            r#"
[allowed]
packages = ["foo"]
"#,
        )
        .unwrap();

        let config = Config::load_from(&path_buf);
        assert!(config.packages.is_empty());
    }

    #[test]
    fn readme_contains_example_config() {
        super::super::util::assert_readme_contains_code_block(
            EXAMPLE_CONFIG.trim().lines(),
            Some("toml"),
        );
    }
}
