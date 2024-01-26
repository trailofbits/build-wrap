use std::process::Command;

#[path = "../src/util.rs"]
mod main_util;
pub use main_util::*;

pub fn build_with_link_wrap() -> Command {
    let link_wrap = env!("CARGO_BIN_EXE_link-wrap");

    let mut command = main_util::cargo_build();
    command.args([
        "--config",
        &format!("target.'cfg(all())'.linker = '{link_wrap}'"),
    ]);

    command
}
