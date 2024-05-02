use std::{path::Path, process::Command};

pub mod util;

#[test]
fn custom_build_name() {
    let dir = Path::new("fixtures/custom_build_name");

    let status = Command::new("cargo")
        .arg("clean")
        .current_dir(dir)
        .status()
        .unwrap();
    assert!(status.success());

    let mut command = util::build_with_build_wrap();
    command.current_dir(dir);

    let output = util::exec_forwarding_output(command, false).unwrap();
    assert!(!output.status.success());

    let stderr = std::str::from_utf8(&output.stderr).unwrap();
    let syscall = if cfg!(target_os = "linux") {
        "socket"
    } else {
        "sendto"
    };
    assert!(
        stderr.contains(&format!("ping: {syscall}: Operation not permitted")),
        "stderr does not contain expected string:\n```\n{stderr}\n```",
    );
}
