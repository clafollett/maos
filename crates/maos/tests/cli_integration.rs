use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_binary_runs() {
    Command::cargo_bin("maos").unwrap().assert().failure(); // Should fail without subcommand
}

#[test]
fn test_help_output() {
    Command::cargo_bin("maos")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Multi-Agent Orchestration System"))
        .stdout(predicate::str::contains("pre-tool-use"))
        .stdout(predicate::str::contains("post-tool-use"))
        .stdout(predicate::str::contains("notify"))
        .stdout(predicate::str::contains("stop"))
        .stdout(predicate::str::contains("subagent-stop"))
        .stdout(predicate::str::contains("user-prompt-submit"))
        .stdout(predicate::str::contains("pre-compact"))
        .stdout(predicate::str::contains("session-start"));
}

#[test]
fn test_version_output() {
    Command::cargo_bin("maos")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("maos"));
}

#[test]
fn test_each_subcommand_help() {
    let commands = [
        "pre-tool-use",
        "post-tool-use",
        "notify",
        "stop",
        "subagent-stop",
        "user-prompt-submit",
        "pre-compact",
        "session-start",
    ];

    for cmd in &commands {
        Command::cargo_bin("maos")
            .unwrap()
            .args([cmd, "--help"])
            .assert()
            .success();
    }
}

#[test]
fn test_stop_help_shows_chat_flag() {
    Command::cargo_bin("maos")
        .unwrap()
        .args(["stop", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--chat"))
        .stdout(predicate::str::contains("Export chat transcript"));
}

#[test]
fn test_user_prompt_submit_help_shows_validate_flag() {
    Command::cargo_bin("maos")
        .unwrap()
        .args(["user-prompt-submit", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--validate"))
        .stdout(predicate::str::contains(
            "Validate prompt before processing",
        ));
}

#[test]
fn test_invalid_command_shows_error() {
    Command::cargo_bin("maos")
        .unwrap()
        .arg("invalid-command")
        .assert()
        .failure()
        .stderr(predicate::str::contains("error"));
}
