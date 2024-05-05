use anyhow::Result;
use std::env::args;

mod linking;
mod util;
mod wrapper;

const DEFAULT_CMD: &str = if cfg!(target_os = "linux") {
    "bwrap
    --ro-bind / /
    --dev-bind /dev /dev
    --bind {OUT_DIR} {OUT_DIR}
    --bind /tmp /tmp
    --unshare-net
    {}"
} else {
    // smoelius: The following blog post is a useful `sandbox-exec` reference:
    // https://7402.org/blog/2020/macos-sandboxing-of-folder.html
    r#"sandbox-exec -p
(version\ 1)\
(deny\ default)\
(allow\ file-read*)\
(allow\ file-write*\ (subpath\ "/dev"))\
(allow\ file-write*\ (subpath\ "{OUT_DIR}"))\
(allow\ file-write*\ (subpath\ "{TMPDIR}"))\
(allow\ file-write*\ (subpath\ "{PRIVATE_TMPDIR}"))\
(allow\ process-exec)\
(allow\ process-fork)\
(allow\ sysctl-read)\
(deny\ network*)
{}"#
};

fn main() -> Result<()> {
    let args: Vec<String> = args().collect();

    if args[1..]
        .iter()
        .all(|arg| matches!(arg.as_str(), "-h" | "--help"))
    {
        help();
        return Ok(());
    }

    linking::link(&args)
}

fn help() {
    println!(
        "{} {}

A linker replacement to help protect against malicious build scripts",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
    );
}
