use crate::util;
use anyhow::{Context, Result};
use assert_cmd::Command;
use cargo_metadata::MetadataCommand;
use std::{
    env::{join_paths, split_paths, var_os},
    ffi::OsString,
    fs::{create_dir_all, write},
    path::PathBuf,
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
        let mut command = Command::cargo_bin("build-wrap").unwrap();
        command.env("HOME", home.path());
        if set_path {
            let metadata = MetadataCommand::new().no_deps().exec().unwrap();
            let target_debug = metadata.target_directory.join("debug").into_std_path_buf();
            command.env("PATH", prepend_path(target_debug).unwrap());
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

fn prepend_path(path: PathBuf) -> Result<OsString> {
    let paths = var_os("PATH").with_context(|| "`PATH` is unset")?;
    let paths_split = split_paths(&paths);
    let paths_chained = std::iter::once(path).chain(paths_split);
    let paths_joined = join_paths(paths_chained)?;
    Ok(paths_joined)
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
