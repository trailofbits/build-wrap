use crate::util;
use std::{io::Write, process::Command};

#[test]
fn edition_2021_toolchain() {
    let result = Command::new("rustup")
        .args(["run", "1.84", "rustc", "--version"])
        .output();
    if !result.is_ok_and(|output| output.status.success()) {
        writeln!(
            std::io::stderr(),
            "Skipping `generated_is_current` test as repository is dirty",
        )
        .unwrap();
        return;
    }

    let temp_package = util::temp_package(Some("tests/build_scripts/ping.rs"), []).unwrap();

    let mut command = util::build_with_build_wrap();
    command.env("RUSTUP_TOOLCHAIN", "1.84");
    command.current_dir(&temp_package);

    let output = util::exec_forwarding_output(command, false).unwrap();
    assert!(
        output.status.success(),
        "`build-wrap` failed with Rust 1.84 toolchain",
    );
}
