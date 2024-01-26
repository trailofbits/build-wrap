mod util;

#[test]
fn ping() {
    let mut command = util::build_with_link_wrap();
    command.current_dir("fixtures/ping");

    let output = util::exec(command, false).unwrap();
    let stderr = std::str::from_utf8(&output.stderr).unwrap();
    assert!(stderr.contains("ping: connect: Network is unreachable"));
}
