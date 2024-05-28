use std::process::{exit, Command, ExitStatus};

fn main() {
    let status: ExitStatus = Command::new("ping")
        .args(["-c", "1", "-v", "127.0.0.1"])
        .status()
        .unwrap();
    let code = status.code().unwrap();
    exit(code);
}
