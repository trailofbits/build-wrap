use crate::util;

#[test]
fn allow() {
    let temp_package = util::temp_package(Some("tests/build_scripts/ping.rs"), []).unwrap();

    for allow in [false, true] {
        let mut command = util::build_with_build_wrap();
        if allow {
            command.env("BUILD_WRAP_ALLOW", "1");
        }
        command.current_dir(&temp_package);

        let output = util::exec_forwarding_output(command, false).unwrap();
        // smoelius: The command should succeed precisely when `BUILD_SCRIPT_ALLOW` is enabled.
        assert_eq!(allow, output.status.success());
        let stderr = std::str::from_utf8(&output.stderr).unwrap();
        assert!(stderr.contains("command failed"));
    }
}
