use std::path::Path;

pub mod util;

#[test]
fn custom_build_name() {
    let dir = Path::new("fixtures/custom_build_name");

    let mut command = util::build_with_build_wrap();
    command.current_dir(dir);

    let output = util::exec(command, false).unwrap();
    assert!(!output.status.success());

    let stderr = std::str::from_utf8(&output.stderr).unwrap();
    assert!(
        stderr.contains("ping: socket: Operation not permitted"),
        "stderr does not contain expected string:\n```\n{stderr}\n```",
    );
}
