//! Tab-bar status script runner (`[ui.tab_status]`).
//!
//! Spawns a non-login shell string, captures the first stdout line, and kills
//! the process on timeout. Presentation wiring lives on `AppState` / the
//! scheduled task loop — this module is the process helper only.

use std::io::{Read, Result as IoResult};
use std::process::Stdio;
use std::time::{Duration, Instant};

/// Fixed kill deadline for a single status command run (not user-configurable in v1).
pub const TAB_STATUS_TIMEOUT: Duration = Duration::from_secs(3);

const POLL_INTERVAL: Duration = Duration::from_millis(25);

#[derive(Debug)]
pub enum TabStatusRunError {
    Spawn(std::io::Error),
    Wait(std::io::Error),
    Timeout,
    Io(std::io::Error),
}

impl std::fmt::Display for TabStatusRunError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Spawn(err) => write!(f, "spawn failed: {err}"),
            Self::Wait(err) => write!(f, "wait failed: {err}"),
            Self::Timeout => write!(f, "timed out"),
            Self::Io(err) => write!(f, "io failed: {err}"),
        }
    }
}

impl std::error::Error for TabStatusRunError {}

/// Run `command` via the platform non-login shell (`sh -c` / `cmd /c`), capture
/// stdout, ignore stderr for display, and return the first line.
///
/// Successful completion (including non-zero exit) returns the truncated-ready
/// first line. Spawn/wait/timeout failures return `Err` so callers keep last-good.
pub fn run_tab_status_command(
    command: &str,
    timeout: Duration,
) -> Result<String, TabStatusRunError> {
    let mut child = crate::platform::tab_status_command_process(command)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(TabStatusRunError::Spawn)?;

    let Some(stdout) = child.stdout.take() else {
        let _ = terminate_and_reap(&mut child);
        return Err(TabStatusRunError::Io(std::io::Error::other(
            "tab status stdout was not piped",
        )));
    };
    let stdout_reader = std::thread::spawn(move || {
        let mut buf = Vec::new();
        let mut reader = stdout;
        reader.read_to_end(&mut buf).map(|_| buf)
    });

    let deadline = Instant::now() + timeout;
    loop {
        match child.try_wait() {
            Ok(Some(_status)) => {
                let stdout = stdout_reader
                    .join()
                    .map_err(|_| TabStatusRunError::Io(std::io::Error::other("stdout reader panicked")))?
                    .map_err(TabStatusRunError::Io)?;
                return Ok(first_line(&stdout));
            }
            Ok(None) => {}
            Err(wait_err) => {
                let _ = terminate_and_reap(&mut child);
                let _ = stdout_reader.join();
                return Err(TabStatusRunError::Wait(wait_err));
            }
        }

        let now = Instant::now();
        if now >= deadline {
            let _ = terminate_and_reap(&mut child);
            let _ = stdout_reader.join();
            return Err(TabStatusRunError::Timeout);
        }

        std::thread::sleep((deadline - now).min(POLL_INTERVAL));
    }
}

fn first_line(stdout: &[u8]) -> String {
    let text = String::from_utf8_lossy(stdout);
    text.lines().next().unwrap_or("").to_string()
}

fn terminate_and_reap(child: &mut std::process::Child) -> IoResult<()> {
    if let Err(kill_err) = child.kill() {
        if child.try_wait()?.is_none() {
            return Err(kill_err);
        }
    }
    child.wait().map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    #[test]
    fn run_tab_status_command_returns_first_line_only() {
        let text = run_tab_status_command("printf 'one\\ntwo\\n'", TAB_STATUS_TIMEOUT)
            .expect("command should succeed");
        assert_eq!(text, "one");
    }

    #[cfg(unix)]
    #[test]
    fn run_tab_status_command_ignores_stderr() {
        let text = run_tab_status_command(
            "printf 'ok\\n' >&1; printf 'err\\n' >&2; exit 1",
            TAB_STATUS_TIMEOUT,
        )
        .expect("non-zero exit with stdout still updates");
        assert_eq!(text, "ok");
    }

    #[cfg(unix)]
    #[test]
    fn run_tab_status_command_times_out_and_kills() {
        let started = Instant::now();
        let err = run_tab_status_command("sleep 10", Duration::from_millis(200))
            .expect_err("slow command should time out");
        assert!(matches!(err, TabStatusRunError::Timeout), "err={err:?}");
        assert!(
            started.elapsed() < Duration::from_secs(2),
            "timeout path should not wait for sleep"
        );
    }

    #[test]
    fn first_line_helper_handles_empty_and_crlf() {
        assert_eq!(first_line(b""), "");
        assert_eq!(first_line(b"hello\r\nworld\r\n"), "hello");
        assert_eq!(first_line(b"only"), "only");
    }
}
