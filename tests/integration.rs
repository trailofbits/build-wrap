use std::{
    env::var_os,
    ffi::OsStr,
    fs::{read_dir, read_to_string},
    path::Path,
};

pub mod util;

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

    let temp_package = util::temp_package(&path).unwrap();

    let mut command = util::build_with_build_wrap();
    // smoelius: `--all-features` to enable optional build dependencies.
    command.arg("--all-features");
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
