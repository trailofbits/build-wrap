use crate::util;
use cargo_metadata::MetadataCommand;
use std::{
    fs::{create_dir_all, write},
    process::Command,
};

#[test]
fn enabled() {
    let home = util::tempdir().unwrap();
    let cargo_home = home.path().join(".cargo");
    create_dir_all(&cargo_home).unwrap();
    write(
        cargo_home.join("config.toml"),
        r#"[target.'cfg(all())']
linker = "build-wrap""#,
    )
    .unwrap();

    for set_path in [false, true] {
        let mut command = Command::new(env!("CARGO_BIN_EXE_build-wrap"));
        command.env("HOME", home.path());
        if set_path {
            let metadata = MetadataCommand::new().no_deps().exec().unwrap();
            let target_debug = metadata.target_directory.join("debug").into_std_path_buf();
            util::prepend_to_path(&mut command, target_debug).unwrap();
        }
        exec_and_check_stdout(
            command,
            &format!(
                "build-wrap is {}",
                if set_path { "ENABLED" } else { "DISABLED" }
            ),
        );
    }
}

#[test]
fn disabled_in_directory() {
    let home = util::tempdir().unwrap();
    let cargo_home = home.path().join(".cargo");
    create_dir_all(&cargo_home).unwrap();
    write(
        cargo_home.join("config.toml"),
        r#"[target.'cfg(all())']
linker = "build-wrap""#,
    )
    .unwrap();

    let metadata = MetadataCommand::new().no_deps().exec().unwrap();
    let target_debug = metadata.target_directory.join("debug").into_std_path_buf();

    let workdir = util::tempdir().unwrap();
    let workdir_path = workdir.path().to_str().unwrap();

    let config_dir = home.path().join(".config/build-wrap");
    create_dir_all(&config_dir).unwrap();
    write(
        config_dir.join("config.toml"),
        format!(
            "\
[allow]
directories = [\"{workdir_path}\"]
"
        ),
    )
    .unwrap();

    for in_allowed_dir in [false, true] {
        let mut command = Command::new(env!("CARGO_BIN_EXE_build-wrap"));
        command.env("HOME", home.path());
        command.env_remove("XDG_CONFIG_HOME");
        util::prepend_to_path(&mut command, target_debug.clone()).unwrap();
        if in_allowed_dir {
            command.current_dir(&workdir);
        }
        exec_and_check_stdout(
            command,
            if in_allowed_dir {
                "build-wrap is ENABLED (but DISABLED in this directory)"
            } else {
                "build-wrap is ENABLED"
            },
        );
    }
}

fn exec_and_check_stdout(mut command: Command, prefix: &str) {
    let output = command.output().unwrap();
    assert!(output.status.success());
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    assert!(
        stdout.lines().any(|line| line.starts_with(prefix)),
        "unexpected stdout: ```\n{stdout}\n```",
    );
}
