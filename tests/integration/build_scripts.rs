use crate::{
    config,
    util::{test_case, TestCase},
};
use std::io::Write;

#[test]
fn build_scripts() {
    config::for_each_test_case("tests/build_scripts", |build_wrap_cmd, path, stderr| {
        #[allow(clippy::explicit_write)]
        writeln!(
            std::io::stderr(),
            "running build script test: {}",
            path.display()
        )
        .unwrap();

        test_case(build_wrap_cmd, &TestCase::BuildScript(path), stderr);

        Ok(())
    })
    .unwrap();
}
