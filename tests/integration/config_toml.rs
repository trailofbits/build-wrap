use crate::util;
use std::fs::{create_dir_all, write};

#[test]
fn config_allow() {
    let temp_package = util::temp_package(Some("tests/build_scripts/ping.rs"), []).unwrap();

    let home = util::tempdir().unwrap();
    let config_build_wrap = home.path().join(".config/build-wrap");
    create_dir_all(&config_build_wrap).unwrap();
    write(config_build_wrap.join("allow.txt"), "temp-package\n").unwrap();

    for allow in [false, true] {
        let mut command = util::build_with_build_wrap();
        command.env_remove("XDG_CONFIG_HOME");
        if allow {
            command.env("HOME", home.path());
        }
        command.current_dir(&temp_package);

        let output = util::exec_forwarding_output(command, false).unwrap();
        // smoelius: The command should succeed precisely when `HOME` is set.
        assert_eq!(allow, output.status.success());
        let stderr = std::str::from_utf8(&output.stderr).unwrap();
        assert!(stderr.contains("command failed"));
    }
}
