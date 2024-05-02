use std::{
    env::var_os,
    ffi::OsStr,
    fs::{read_dir, read_to_string, DirEntry},
    io::Write,
    path::Path,
};

pub mod util;

#[test]
fn integration() {
    let test_cases = Path::new("tests/cases");

    if let Some(testname) = var_os("TESTNAME") {
        test_case(&test_cases.join(testname).with_extension("rs"));
    } else {
        let mut entries = read_dir("tests/cases")
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        entries.sort_by_key(DirEntry::path);
        for entry in entries {
            let path = entry.path();

            if path.extension() != Some(OsStr::new("rs")) {
                continue;
            }

            #[allow(clippy::explicit_write)]
            writeln!(std::io::stderr(), "{}", path.display()).unwrap();

            test_case(&path);
        }
    }
}

fn test_case(path: &Path) {
    let mut stderr_path = path.with_extension("stderr");
    if !stderr_path.exists() {
        stderr_path = if cfg!(target_os = "linux") {
            path.with_extension("linux.stderr")
        } else {
            path.with_extension("macos.stderr")
        }
    }
    let expected_stderr_substring = read_to_string(stderr_path).unwrap();

    let temp_package = util::temp_package(path).unwrap();

    let mut command = util::build_with_build_wrap();
    // smoelius: `--all-features` to enable optional build dependencies.
    command.arg("--all-features");
    command.current_dir(&temp_package);

    let output = util::exec_forwarding_output(command, false).unwrap();
    assert_eq!(
        expected_stderr_substring.is_empty(),
        output.status.success(),
        "{path:?} failed in {:?}",
        temp_package.into_path()
    );

    let stderr = std::str::from_utf8(&output.stderr).unwrap();
    assert!(
        stderr.contains(expected_stderr_substring.trim_end()),
        "`{}` stderr does not contain expected string:\n```\n{stderr}\n```",
        path.display(),
    );
}
