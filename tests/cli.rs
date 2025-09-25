use assert_cmd::Command;
use predicates::prelude::*;

const FILE_PATH: &str = "examples/hello_world.wdl";

// Helper function to create a test command
fn cmd() -> Command {
    Command::cargo_bin("wdlparse").unwrap()
}

#[test]
fn test_parse_command_human_format() {
    cmd()
        .arg("parse")
        .arg(FILE_PATH)
        .arg("--format")
        .arg("human")
        .assert()
        .success()
        .stdout(predicate::str::contains("Parsed:"));
}

#[test]
fn test_parse_command_tree_format() {
    cmd()
        .arg("parse")
        .arg(FILE_PATH)
        .arg("--format")
        .arg("tree")
        .assert()
        .success()
        .stdout(predicate::str::contains("Syntax Tree:"));
}

#[test]
fn test_parse_command_json_format() {
    cmd()
        .arg("parse")
        .arg(FILE_PATH)
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"file\""));
}

#[test]
fn test_info_command() {
    cmd()
        .arg("info")
        .arg(FILE_PATH)
        .assert()
        .success()
        .stdout(predicate::str::contains("WDL File Info:"))
        .stdout(predicate::str::contains("say_hello"))
        .stdout(predicate::str::contains("hello_world"));
}

#[test]
fn test_nonexistent_file() {
    cmd().arg("parse").arg("nonexistent.wdl").assert().failure();
}

#[test]
fn test_help_message() {
    cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("wdlparse"))
        .stdout(predicate::str::contains("parse"))
        .stdout(predicate::str::contains("info"));
}
