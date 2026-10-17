//! Binary startup behavior that does not require a terminal.

use std::process::{Command, Stdio};

#[test]
fn non_tty_startup_is_rejected_with_a_clear_error() {
    let output = Command::new(env!("CARGO_BIN_EXE_randloom"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("spawn randloom binary");

    assert!(
        !output.status.success(),
        "non-TTY startup must fail: status={:?}",
        output.status
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("interactive terminal"),
        "unexpected stderr: {stderr:?}"
    );
}
