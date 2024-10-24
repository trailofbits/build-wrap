use crate::util::{test_case, TestCase, ToUtf8};
use anyhow::{ensure, Result};
use assert_cmd::Command;
use std::{
    env::var_os,
    ffi::OsStr,
    fs::{read_dir, read_to_string, DirEntry},
    io::Write,
    path::{Path, PathBuf},
};

#[test]
fn third_party() {
    warn_if_go_build_exists();

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
        writeln!(
            std::io::stderr(),
            "running third-party test: {}",
            path.display()
        )
        .unwrap();

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

fn warn_if_go_build_exists() {
    let Ok(go_build_path) = go_build_path() else {
        return;
    };

    if go_build_path.try_exists().unwrap_or(false) {
        #[allow(clippy::explicit_write)]
        writeln!(
            std::io::stderr(),
            "`go-build` exists at `{}`; some third-party tests may fail!",
            go_build_path.display()
        )
        .unwrap();
    }
}

fn go_build_path() -> Result<PathBuf> {
    let output = Command::new("go").args(["env", "GOCACHE"]).output()?;
    ensure!(output.status.success());
    let stdout = std::str::from_utf8(&output.stdout)?;
    Ok(PathBuf::from(stdout.trim_end()))
}
