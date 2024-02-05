pub mod util;

#[test]
fn dogfood() {
    let mut command = util::build_with_build_wrap();
    command.env("BUILD_WRAP_CMD", "time -p");

    let output = util::exec(command, true).unwrap();
    let stderr = std::str::from_utf8(&output.stderr).unwrap();
    let lines = stderr.lines().collect::<Vec<_>>();
    assert!(
        lines.windows(3).any(|w| {
            assert_eq!(3, w.len());
            w[0].contains("] real ") && w[1].contains("] user ") && w[2].contains("] sys")
        }),
        "failed to find `time` output"
    );
}
