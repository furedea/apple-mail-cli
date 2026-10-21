use std::error::Error;
use std::fmt::{self, Display, Formatter};

pub trait MailBridge {
    /// Executes one serialized request and returns one serialized response envelope.
    ///
    /// # Errors
    ///
    /// Returns an error when the bridge cannot start, times out, fails, or emits invalid bytes.
    fn execute(&mut self, request: &str) -> Result<String, BridgeError>;
}

#[derive(Debug)]
pub enum BridgeError {
    Unavailable(String),
    Timeout,
    Failed { code: Option<i32>, stderr: String },
    InvalidOutput(String),
}

impl Display for BridgeError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write_bridge_error(self, formatter)
    }
}

impl Error for BridgeError {}

fn write_bridge_error(error: &BridgeError, formatter: &mut Formatter<'_>) -> fmt::Result {
    match error {
        BridgeError::Unavailable(message) => write!(formatter, "bridge unavailable: {message}"),
        BridgeError::Timeout => formatter.write_str("bridge timed out"),
        BridgeError::Failed { code, stderr } => {
            write!(formatter, "bridge failed with exit code {code:?}: {stderr}")
        }
        BridgeError::InvalidOutput(message) => {
            write!(formatter, "bridge returned invalid output: {message}")
        }
    }
}
