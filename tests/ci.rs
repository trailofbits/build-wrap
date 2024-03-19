use assert_cmd::Command;

pub mod util;

#[test]
fn clippy() {
    Command::new("cargo")
        .args([
            "+nightly",
            "clippy",
            "--all-features",
            "--all-targets",
            "--",
            "--deny=warnings",
        ])
        .assert()
        .success();
}
