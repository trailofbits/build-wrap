use crate::util;
use std::{path::Path, process::Command};

const DIR: &str = "fixtures/cargo_config";

#[test]
fn cargo_config() {
    let command = util::build_with_default_linker();
    test_build(command);

    // smoelius: When `build-wrap` builds the wrapper package, it expects the target directory to be
    // `target`. So building the wrapper package in `fixtures/cargo_config` would fail because it
    // contains a `.cargo/config.toml` that sets the target directory to `target-custom`.

    // smoelius: The build script in `fixtures/cargo_config` prints the path of the current
    // executable, i.e., the wrapped, original build script. Previously, this was unpacked into
    // `std::env::temp_dir()`. However, `build-wrap` now renames the original build script so that
    // it is a sibling of the wrapper build script. Hence, when this test is run, the current
    // executable should be in `target-custom` alongside the wrapper build script.
    let command = util::build_with_build_wrap();
    test_build(command);

    // smoelius: Building the wrapper package should similarly succeed if `CARGO_TARGET_DIR` is set.
    // Note that the `fixtures/cargo_config` directory must be cleaned for it to be rebuilt.
    let status = Command::new("cargo")
        .arg("clean")
        .current_dir(DIR)
        .status()
        .unwrap();
    assert!(status.success());

    let mut command = util::build_with_build_wrap();
    command.env("CARGO_TARGET_DIR", "target-custom");
    test_build(command);
}

fn test_build(mut command: Command) {
    command.current_dir(DIR);
    let output = util::exec_forwarding_output(command, true).unwrap();
    let stderr = std::str::from_utf8(&output.stderr).unwrap();
    let expected_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(DIR)
        .join("target-custom/debug");
    assert!(stderr.lines().any(|line| line.starts_with(&format!(
        "warning: cargo_config@0.1.0: {}/",
        trim_trailing_slashes(&expected_dir.to_string_lossy())
    ))));
}

fn trim_trailing_slashes(s: &str) -> &str {
    s.trim_end_matches('/')
}
