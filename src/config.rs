use std::{
    env,
    fs::read_to_string,
    path::{Path, PathBuf},
    sync::LazyLock,
};

static CONFIG: LazyLock<Config> = LazyLock::new(Config::load);

#[derive(Default)]
struct Config {
    directories: Vec<PathBuf>,
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
                eprintln!("warning: {}: unrecognized table `[{key}]`", path.display());
            }
        }

        let mut directories = Vec::new();
        let mut packages = Vec::new();

        for section in ["allow", "ignore"] {
            if let Some(table) = table.get(section).and_then(toml::Value::as_table) {
                extend_directories(&mut directories, table.get("directories"));
                extend_from_string_array(&mut packages, table.get("packages"));
            }
        }

        Self {
            directories,
            packages,
        }
    }
}

fn extend_directories(vec: &mut Vec<PathBuf>, value: Option<&toml::Value>) {
    if let Some(array) = value.and_then(toml::Value::as_array) {
        for item in array {
            if let Some(s) = item.as_str() {
                vec.push(expand_tilde(s));
            }
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

fn expand_tilde(s: &str) -> PathBuf {
    expand_tilde_with_home(s, home::home_dir())
}

fn expand_tilde_with_home(s: &str, home: Option<PathBuf>) -> PathBuf {
    if s == "~" {
        return home.unwrap_or_else(|| PathBuf::from(s));
    }

    if let Some(suffix) = s.strip_prefix("~/")
        && let Some(home) = home
    {
        return home.join(suffix);
    }

    PathBuf::from(s)
}

pub fn directory_allowed(path: &Path) -> bool {
    CONFIG.directories.iter().any(|d| path.starts_with(d))
}

pub fn package_allowed() -> bool {
    let Ok(name) = env::var("CARGO_PKG_NAME") else {
        return false;
    };
    CONFIG.packages.iter().any(|p| p == &name)
}

#[cfg(test)]
#[allow(clippy::disallowed_methods)]
mod test {
    use super::*;
    use std::{fs::write, path::PathBuf};

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
            vec![
                PathBuf::from("/home/user/project-a"),
                PathBuf::from("/home/user/project-b")
            ]
        );
        assert_eq!(config.packages, vec!["aws-lc-fips-sys", "svm-rs-builds"]);
    }

    #[test]
    fn expand_tilde_expands_home_dir() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        assert_eq!(expand_tilde_with_home("~", Some(home.clone())), home);
        assert_eq!(
            expand_tilde_with_home("~/project", Some(home.clone())),
            home.join("project")
        );
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
        assert_eq!(config.directories, vec![PathBuf::from("/tmp/myproject")]);
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
