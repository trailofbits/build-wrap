use std::process::Command;

pub mod util;

#[test]
fn build_wrap_cmd_changed() {
    let temp_package =
        util::temp_package("tests/cases/rerun_if_build_wrap_cmd_changed.rs").unwrap();

    // smoelius: Build with default `BUILD_WRAP_CMD`.

    let mut command = util::build_with_build_wrap();
    command.current_dir(&temp_package);

    exec_and_check_stdout(command, false, "] real ");

    // smoelius: Build with `BUILD_WRAP_CMD` set to `time -p {}`.

    let mut command = util::build_with_build_wrap();
    command.env("BUILD_WRAP_CMD", "time -p {}");
    command.current_dir(&temp_package);

    exec_and_check_stdout(command, false, "] real ");

    // smoelius: Clean and build again with `BUILD_WRAP_CMD` set to `time -p {}`.

    let status = Command::new("cargo")
        .arg("clean")
        .current_dir(&temp_package)
        .status()
        .unwrap();
    assert!(status.success());

    let mut command = util::build_with_build_wrap();
    command.env("BUILD_WRAP_CMD", "time -p {}");
    command.current_dir(&temp_package);

    exec_and_check_stdout(command, true, "] real ");
}

fn exec_and_check_stdout(command: Command, should_contain: bool, needle: &str) {
    let output = util::exec(command, false).unwrap();
    assert!(output.status.success());

    let stderr = std::str::from_utf8(&output.stderr).unwrap();
    let contains = stderr.contains(needle);
    assert_eq!(should_contain, contains);
}
