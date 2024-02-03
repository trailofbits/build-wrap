use anyhow::Result;
use std::{
    env::var_os,
    ffi::OsStr,
    fs::{copy, create_dir, read_dir, read_to_string, write},
    path::Path,
};
use tempfile::{tempdir, TempDir};

mod util;

#[test]
fn integration() {
    let test_cases = Path::new("tests/cases");

    if let Some(testname) = var_os("TESTNAME") {
        test_case(&test_cases.join(testname).with_extension("rs"));
    } else {
        for entry in read_dir("tests/cases").unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();

            if path.extension() != Some(OsStr::new("rs")) {
                continue;
            }

            test_case(&path);
        }
    }
}

fn test_case(path: &Path) {
    let stderr_path = path.with_extension("stderr");
    let expected_stderr_substring = read_to_string(stderr_path).unwrap();

    let temp_package = temp_package(&path).unwrap();

    let mut command = util::build_with_build_wrap();
    command.current_dir(&temp_package);

    let output = util::exec(command, false).unwrap();
    assert_eq!(
        expected_stderr_substring.is_empty(),
        output.status.success(),
        "{path:?} failed in {:?}",
        temp_package.into_path()
    );

    let stderr = std::str::from_utf8(&output.stderr).unwrap();
    assert!(stderr.contains(expected_stderr_substring.trim_end()));
}

fn temp_package(build_script_path: &Path) -> Result<TempDir> {
    let tempdir = tempdir()?;

    write(tempdir.path().join("Cargo.toml"), CARGO_TOML)?;
    copy(build_script_path, tempdir.path().join("build.rs"))?;
    create_dir(tempdir.path().join("src"))?;
    write(tempdir.path().join("src/lib.rs"), "")?;

    Ok(tempdir)
}

const CARGO_TOML: &str = r#"
[package]
name = "temp-package"
version = "0.1.0"
edition = "2021"
publish = false
"#;
