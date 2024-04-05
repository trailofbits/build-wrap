use std::{env::temp_dir, path::Path, process::Command};

pub mod util;

const DIR: &str = "fixtures/cargo_config";

#[test]
fn cargo_config() {
    let command = util::build_with_default_linker();
    test_build(
        command,
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(DIR)
            .join("target-custom/debug"),
    );

    // smoelius: When `build-wrap` builds the wrapper package, it expects the target directory to be
    // `target`. So building the wrapper package in `fixtures/cargo_config` would fail because it
    // contains a `.cargo/config.toml` that sets the target directory to `target-custom`.
    let command = util::build_with_build_wrap();
    test_build(command, &temp_dir());
}

fn test_build(mut command: Command, expected_dir: &Path) {
    command.current_dir(DIR);
    let output = util::exec(command, true).unwrap();
    let stderr = std::str::from_utf8(&output.stderr).unwrap();
    assert!(stderr.lines().any(|line| line.starts_with(&format!(
        "warning: cargo_config@0.1.0: {}/",
        trim_trailing_slashes(&expected_dir.to_string_lossy())
    ))));
}

fn trim_trailing_slashes(s: &str) -> &str {
    s.trim_end_matches('/')
}
