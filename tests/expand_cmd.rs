use anyhow::Result;
use std::{env::set_var, path::Path};

pub mod util;

#[test]
fn expand_cmd() {
    set_var("KEY", "VALUE");

    let successes = [
        ("left path right", "{}"),
        ("left VALUE right", "{KEY}"),
        ("left { right", "{{"),
        ("left } right", "}}"),
    ];

    let failures = [
        ("environment variable `UNKNOWN` not found", "{UNKNOWN}"),
        ("unbalanced '{'", "{"),
        ("unbalanced '}'", "}"),
    ];

    for (expected, s) in successes {
        assert_eq!(expected, surround_and_expand(s).unwrap());
    }

    for (expected, s) in failures {
        assert_eq!(expected, surround_and_expand(s).unwrap_err().to_string());
    }
}

fn surround_and_expand(s: &str) -> Result<String> {
    let cmd = String::from("left ") + s + " right";
    util::__expand_cmd(&cmd, Path::new("path"))
}
