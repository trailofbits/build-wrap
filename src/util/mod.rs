use std::{env::var_os, ffi::OsString, process::Command};

mod common;
pub use common::{exec, ToUtf8};

// smoelius: `__expand_cmd` is not meant to be used outside of this module. See the comment
// preceding `__expand_cmd`.
#[allow(unused_imports)]
pub use common::__expand_cmd;

#[must_use]
pub fn cargo_build() -> Command {
    // smoelius: Respect `CARGO` environment variable, if set.
    let cargo = var_os("CARGO").unwrap_or(OsString::from("cargo"));

    let mut command = Command::new(cargo);
    command.arg("build");

    // smoelius: Show build script (e.g., wrapper) output.
    // See: https://github.com/rust-lang/cargo/issues/985#issuecomment-258311111
    command.arg("-vv");

    // smoelius: Show linker output.
    // See: https://stackoverflow.com/a/71866183
    command.env("RUSTC_LOG", "rustc_codegen_ssa::back::link=info");

    command
}
