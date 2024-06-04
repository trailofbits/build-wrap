use crate::util::{test_case, TestCase};
use std::{
    env::var_os,
    ffi::OsStr,
    fs::{read_dir, DirEntry},
    io::Write,
};

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
        writeln!(
            std::io::stderr(),
            "running build script test: {}",
            path.display()
        )
        .unwrap();

        test_case(
            &TestCase::BuildScript(&path),
            &path.with_extension("stderr"),
        );
    }
}
