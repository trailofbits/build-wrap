use snapbox::{assert_data_eq, Data};
use std::{
    env::var_os,
    ffi::OsStr,
    fs::{read_dir, read_to_string, DirEntry},
    io::Write,
    path::Path,
};

pub mod util;
use util::ToUtf8;

#[test]
fn build_scripts() {
    let mut entries = read_dir("tests/build_scripts")
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    entries.sort_by_key(DirEntry::path);
    for entry in entries {
        let path = entry.path();

        if path.extension() != Some(OsStr::new("rs")) {
            continue;
        }

        if let Some(testname) = var_os("TESTNAME") {
            if path.file_stem() != Some(&testname) {
                continue;
            }
        }

        #[allow(clippy::explicit_write)]
        writeln!(std::io::stderr(), "{}", path.display()).unwrap();

        test_case(
            &TestCase::BuildScript(&path),
            &path.with_extension("stderr"),
        );
    }
}

#[test]
fn third_party() {
    let mut entries = read_dir("tests/third_party")
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    entries.sort_by_key(DirEntry::path);
    for entry in entries {
        let path = entry.path();

        if path.extension() != Some(OsStr::new("txt")) {
            continue;
        }

        if let Some(testname) = var_os("TESTNAME") {
            if path.file_stem() != Some(&testname) {
                continue;
            }
        }

        #[allow(clippy::explicit_write)]
        writeln!(std::io::stderr(), "{}", path.display()).unwrap();

        let file_stem = path.file_stem().unwrap();
        let name = file_stem.to_utf8().unwrap();

        let version = parse_version_file(&path);

        test_case(
            &TestCase::ThirdParty(name, &version),
            &path.with_extension("stderr"),
        );
    }
}

fn parse_version_file(path: &Path) -> String {
    let contents = read_to_string(path).unwrap();

    contents
        .lines()
        .map(|line| {
            let i = line.find('#').unwrap_or(line.len());
            &line[..i]
        })
        .collect::<Vec<_>>()
        .join("")
        .trim()
        .to_owned()
}

#[derive(Debug)]
enum TestCase<'a> {
    BuildScript(&'a Path),
    ThirdParty(&'a str, &'a str),
}

fn test_case(test_case: &TestCase, stderr_path: &Path) {
    let mut stderr_path = stderr_path.to_path_buf();
    if !stderr_path.exists() {
        stderr_path = if cfg!(target_os = "linux") {
            stderr_path.with_extension("linux.stderr")
        } else {
            stderr_path.with_extension("macos.stderr")
        }
    }
    let expected_stderr = read_to_string(&stderr_path).unwrap();

    let temp_package = match *test_case {
        TestCase::BuildScript(path) => util::temp_package(Some(path), []),
        TestCase::ThirdParty(name, version) => util::temp_package(None::<&Path>, [(name, version)]),
    }
    .unwrap();

    let mut command = util::build_with_build_wrap();
    // smoelius: `--all-features` to enable optional build dependencies.
    command.arg("--all-features");
    command.current_dir(&temp_package);

    let output = util::exec_forwarding_output(command, false).unwrap();
    assert_eq!(
        expected_stderr.is_empty(),
        output.status.success(),
        "{test_case:?} failed in {:?}",
        temp_package.into_path()
    );

    if expected_stderr.is_empty() {
        return;
    }

    let stderr_actual = std::str::from_utf8(&output.stderr).unwrap();
    assert_data_eq!(stderr_actual, Data::read_from(&stderr_path, None));
}
