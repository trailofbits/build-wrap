//! This file must remain in the tests folder to ensure that `CARGO_BIN_EXE_build-wrap` is defined
//! at compile time.
//! See: https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-crates

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
