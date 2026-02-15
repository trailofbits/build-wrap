use std::process::{Command, ExitStatus, exit};

const DEADLINE_FLAG: &str = if cfg!(target_os = "linux") {
    "-w"
} else {
    "-t"
};

fn main() {
    let status: ExitStatus = Command::new("ping")
        .args(["-c", "1", DEADLINE_FLAG, "1", "-v", "127.0.0.1"])
        .status()
        .unwrap();
    let code = status.code().unwrap();
    exit(code);
}
