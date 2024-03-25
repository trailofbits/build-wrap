fn main() {
    println!(
        "cargo:warning={}",
        std::env::current_exe().unwrap().display()
    );
}
