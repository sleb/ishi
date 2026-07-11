#![allow(dead_code)]

use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

/// Scaffolds a real PARA workspace at `dir` without going through the `ishi`
/// binary, so tests that aren't exercising `init` itself can set up their
/// fixture cheaply.
pub fn init_workspace(dir: &Path) {
    ishi::workspace::init(dir).expect("failed to init workspace");
}

/// Writes an executable shell script at `dir/name` to stand in for
/// `$EDITOR`. `body` is the script's shell code; the file it's invoked on is
/// available as `$1`.
pub fn write_fake_editor(dir: &Path, name: &str, body: &str) -> PathBuf {
    let path = dir.join(name);
    fs::write(&path, format!("#!/bin/sh\n{body}\n")).expect("failed to write fake editor");
    let mut perms = fs::metadata(&path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&path, perms).unwrap();
    path
}

/// Runs the real `ishi` binary with `args` in `dir`. `editor`, if set,
/// becomes `$EDITOR`; otherwise `$EDITOR` is unset. `stdin`, if set, is
/// written to the child's stdin and then closed (EOF), for tests that need
/// to answer a `Ui::confirm` prompt.
pub fn ishi(args: &[&str], dir: &Path, editor: Option<&Path>, stdin: Option<&str>) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_ishi"));
    cmd.args(args)
        .current_dir(dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    match editor {
        Some(editor) => cmd.env("EDITOR", editor),
        None => cmd.env_remove("EDITOR"),
    };

    let mut child = cmd.spawn().expect("failed to spawn ishi");

    if let Some(input) = stdin {
        let mut child_stdin = child.stdin.take().expect("stdin was piped");
        child_stdin
            .write_all(input.as_bytes())
            .expect("failed to write to ishi's stdin");
    }

    child
        .wait_with_output()
        .expect("failed to wait on ishi's output")
}

/// Runs the real `ishi` binary with `args` in `dir`, with `$HOME` pointed at
/// `home` and `$EDITOR` unset. For tests exercising `-g`/`--global` behavior
/// that needs a controlled home directory rather than the real one.
pub fn ishi_with_home(args: &[&str], dir: &Path, home: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ishi"))
        .args(args)
        .current_dir(dir)
        .env("HOME", home)
        .env_remove("EDITOR")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("failed to run ishi")
}

/// Runs the real `ishi` binary with `args` in `dir`, with `$HOME` pointed at
/// `home` and `$EDITOR` pointed at `editor`. For tests exercising `-g`
/// behavior that also needs to observe/control the editor invocation.
pub fn ishi_with_home_and_editor(args: &[&str], dir: &Path, home: &Path, editor: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ishi"))
        .args(args)
        .current_dir(dir)
        .env("HOME", home)
        .env("EDITOR", editor)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("failed to run ishi")
}

pub fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout was not valid utf-8")
}

pub fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("stderr was not valid utf-8")
}
