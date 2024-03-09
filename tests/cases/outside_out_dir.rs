use std::{env::var_os, fs::write, path::PathBuf};

fn main() {
    let out_dir = var_os("OUT_DIR").unwrap();
    let path = PathBuf::from(out_dir).parent().unwrap().to_path_buf();
    write(path.join("OUTSIDE_OUT_DIR"), "x").unwrap();
}
