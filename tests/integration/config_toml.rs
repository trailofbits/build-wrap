use crate::util;
use std::fs::{create_dir_all, write};

#[test]
fn config_toml_allow_packages() {
    config_toml_packages(
        "\
[allow]
packages = [\"temp-package\"]
",
    );
}

#[test]
fn config_toml_ignore_packages() {
    config_toml_packages(
        "\
[ignore]
packages = [\"temp-package\"]
",
    );
}

fn config_toml_packages(config_contents: &str) {
    let home = util::tempdir().unwrap();
    let config_dir = home.path().join(".config/build-wrap");
    create_dir_all(&config_dir).unwrap();
    write(config_dir.join("config.toml"), config_contents).unwrap();

    // Each iteration needs a fresh temp_package because the config check happens at link time.
    // Cargo caches the linked build script, so a second `cargo build` in the same directory
    // would reuse the artifact from the first iteration.
    for allowed in [false, true] {
        let temp_package = util::temp_package(Some("tests/build_scripts/ping.rs"), []).unwrap();

        let mut command = util::build_with_build_wrap();
        command.env_remove("XDG_CONFIG_HOME");
        if allowed {
            command.env("HOME", home.path());
        }
        command.current_dir(&temp_package);

        let output = util::exec_forwarding_output(command, false).unwrap();
        assert_eq!(allowed, output.status.success());
        if !allowed {
            let stderr = std::str::from_utf8(&output.stderr).unwrap();
            assert!(stderr.contains("command failed"));
        }
    }
}

#[test]
fn config_toml_allow_directories() {
    config_toml_directories("allow");
}

#[test]
fn config_toml_ignore_directories() {
    config_toml_directories("ignore");
}

fn config_toml_directories(section: &str) {
    let home = util::tempdir().unwrap();
    let config_dir = home.path().join(".config/build-wrap");
    create_dir_all(&config_dir).unwrap();

    // Each iteration needs a fresh temp_package (see comment in `config_toml_packages`).
    for allowed in [false, true] {
        let temp_package = util::temp_package(Some("tests/build_scripts/ping.rs"), []).unwrap();

        let temp_package_path = temp_package.path().to_str().unwrap();
        let config_contents = format!(
            "\
[{section}]
directories = [\"{temp_package_path}\"]
"
        );
        write(config_dir.join("config.toml"), &config_contents).unwrap();

        let mut command = util::build_with_build_wrap();
        command.env_remove("XDG_CONFIG_HOME");
        if allowed {
            command.env("HOME", home.path());
        }
        command.current_dir(&temp_package);

        let output = util::exec_forwarding_output(command, false).unwrap();
        assert_eq!(allowed, output.status.success());
        if !allowed {
            let stderr = std::str::from_utf8(&output.stderr).unwrap();
            assert!(stderr.contains("command failed"));
        }
    }
}
