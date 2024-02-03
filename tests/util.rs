use std::process::Command;

#[path = "../src/util.rs"]
mod main_util;
pub use main_util::*;

#[must_use]
pub fn build_with_build_wrap() -> Command {
    let build_wrap = env!("CARGO_BIN_EXE_build-wrap");

    let mut command = main_util::cargo_build();
    command.args([
        "--config",
        &format!("target.'cfg(all())'.linker = '{build_wrap}'"),
    ]);

    command
}
