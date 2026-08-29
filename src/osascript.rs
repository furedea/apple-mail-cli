use std::io::Read;
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use crate::bridge::{BridgeError, MailBridge};

const JXA_SOURCE: &str = include_str!("mail_bridge.js");
const OSASCRIPT_PATH: &str = "/usr/bin/osascript";
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
const POLL_INTERVAL: Duration = Duration::from_millis(10);
const MAX_STDOUT_BYTES: u64 = 4_000_000;
const MAX_STDERR_BYTES: u64 = 65_536;
const MAX_STDERR_CHARS: usize = 4_096;

pub struct OsascriptBridge {
    timeout: Duration,
}

impl OsascriptBridge {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            timeout: DEFAULT_TIMEOUT,
        }
    }
}

impl Default for OsascriptBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl MailBridge for OsascriptBridge {
    fn execute(&mut self, request: &str) -> Result<String, BridgeError> {
        let output = run_process(
            Path::new(OSASCRIPT_PATH),
            &arguments_for(request),
            self.timeout,
        )?;
        if !output.status.success() {
            return Err(BridgeError::Failed {
                code: output.status.code(),
                stderr: bounded_lossy_text(&output.stderr),
            });
        }

        String::from_utf8(output.stdout)
            .map_err(|error| BridgeError::InvalidOutput(error.to_string()))
    }
}

fn arguments_for(request: &str) -> [&str; 6] {
    ["-l", "JavaScript", "-e", JXA_SOURCE, "--", request]
}

fn run_process(
    program: &Path,
    arguments: &[&str],
    timeout: Duration,
) -> Result<ProcessOutput, BridgeError> {
    let mut child = Command::new(program)
        .args(arguments)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| BridgeError::Unavailable(error.to_string()))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| BridgeError::Unavailable("failed to capture stdout".to_owned()))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| BridgeError::Unavailable("failed to capture stderr".to_owned()))?;
    let stdout_reader = read_in_background(stdout, MAX_STDOUT_BYTES);
    let stderr_reader = read_in_background(stderr, MAX_STDERR_BYTES);
    let started_at = Instant::now();

    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                return collect_output(status, stdout_reader, stderr_reader);
            }
            Ok(None) if started_at.elapsed() >= timeout => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = collect_read(stdout_reader);
                let _ = collect_read(stderr_reader);
                return Err(BridgeError::Timeout);
            }
            Ok(None) => thread::sleep(POLL_INTERVAL),
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = collect_read(stdout_reader);
                let _ = collect_read(stderr_reader);
                return Err(BridgeError::Unavailable(error.to_string()));
            }
        }
    }
}

fn read_in_background(
    stream: impl Read + Send + 'static,
    max_bytes: u64,
) -> JoinHandle<std::io::Result<Vec<u8>>> {
    thread::spawn(move || {
        let mut bytes = Vec::new();
        stream.take(max_bytes + 1).read_to_end(&mut bytes)?;
        Ok(bytes)
    })
}

fn collect_output(
    status: ExitStatus,
    stdout_reader: JoinHandle<std::io::Result<Vec<u8>>>,
    stderr_reader: JoinHandle<std::io::Result<Vec<u8>>>,
) -> Result<ProcessOutput, BridgeError> {
    let stdout = collect_read(stdout_reader)?;
    let stderr = collect_read(stderr_reader)?;
    if u64::try_from(stdout.len()).unwrap_or(u64::MAX) > MAX_STDOUT_BYTES {
        return Err(BridgeError::InvalidOutput(
            "stdout exceeded the supported limit".to_owned(),
        ));
    }
    if u64::try_from(stderr.len()).unwrap_or(u64::MAX) > MAX_STDERR_BYTES {
        return Err(BridgeError::InvalidOutput(
            "stderr exceeded the supported limit".to_owned(),
        ));
    }
    Ok(ProcessOutput {
        status,
        stdout,
        stderr,
    })
}

fn collect_read(reader: JoinHandle<std::io::Result<Vec<u8>>>) -> Result<Vec<u8>, BridgeError> {
    reader
        .join()
        .map_err(|_| BridgeError::Unavailable("output reader panicked".to_owned()))?
        .map_err(|error| BridgeError::Unavailable(error.to_string()))
}

fn bounded_lossy_text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes)
        .chars()
        .take(MAX_STDERR_CHARS)
        .collect()
}

struct ProcessOutput {
    status: ExitStatus,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::time::Duration;

    use crate::bridge::BridgeError;

    use super::*;

    #[test]
    fn request_is_passed_as_an_argument_instead_of_jxa_source() {
        let request = r#"{"action":"search","query":"'; dangerous()"}"#;

        let arguments = arguments_for(request);

        assert_eq!(arguments[0], "-l");
        assert_eq!(arguments[1], "JavaScript");
        assert_eq!(arguments[2], "-e");
        assert_eq!(arguments[4], "--");
        assert_eq!(arguments[5], request);
        assert!(!arguments[3].contains(request));
    }

    #[test]
    fn process_is_terminated_after_its_timeout() {
        let result = run_process(Path::new("/bin/sleep"), &["1"], Duration::from_millis(10));

        assert!(matches!(result, Err(BridgeError::Timeout)));
    }

    #[test]
    fn process_output_above_the_limit_is_rejected() {
        let result = run_process(Path::new("/usr/bin/yes"), &[], Duration::from_secs(1));

        assert!(matches!(result, Err(BridgeError::InvalidOutput(_))));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn message_mailbox_paths_are_relative_to_their_accounts() {
        const HARNESS: &str = r#"
run = function () {
  const account = {
    id: function () { return "account-1"; },
    name: function () { return "Exchange"; },
  };
  const parentMailbox = {
    id: function () { return "mailbox-1"; },
    name: function () { return "Projects"; },
    account: function () { return account; },
    container: function () { return account; },
  };
  const mailbox = {
    name: function () { return "Inbox"; },
    account: function () { return account; },
    container: function () { return parentMailbox; },
  };
  return mailboxPath(mailbox);
};
"#;
        let source = format!("{JXA_SOURCE}\n{HARNESS}");

        let output = run_process(
            Path::new(OSASCRIPT_PATH),
            &["-l", "JavaScript", "-e", &source],
            DEFAULT_TIMEOUT,
        )
        .expect("JXA path harness should run");

        assert!(output.status.success());
        assert_eq!(
            String::from_utf8(output.stdout)
                .expect("JXA output should be UTF-8")
                .trim(),
            "/Projects/Inbox",
        );
    }
}
