use std::{env, path::Path, process::Command};

fn main() {
    if cfg!(not(target_os = "linux")) {
        return;
    }
    let out_dir = env::var("OUT_DIR").unwrap();
    let exe_path = Path::new(&out_dir).join("sandboxer");
    let mut command = Command::new("cc");
    command.args(["assets/sandboxer.c", "-o", &exe_path.to_string_lossy()]);
    let status = command.status().unwrap();
    assert!(status.success());
}
