use assert_cmd::Command;

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
