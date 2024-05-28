use std::process::{exit, Command, ExitStatus, Stdio};

fn main() {
    let status: ExitStatus = Command::new("echo").stdout(Stdio::null()).status().unwrap();
    let code = status.code().unwrap();
    exit(code);
}
