use std::{env::var_os, fs::write, path::PathBuf};

fn main() {
    let out_dir = var_os("OUT_DIR").unwrap();
    let path = PathBuf::from(out_dir);
    write(path.join("INSIDE_OUT_DIR"), []).unwrap();
}
