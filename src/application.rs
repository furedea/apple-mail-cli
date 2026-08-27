use std::error::Error;
use std::fmt::{self, Display, Formatter};

use serde_json::Value;

use crate::bridge::{BridgeError, MailBridge};
use crate::cli::Command;
use crate::request::MailRequest;

/// Executes a parsed command through a Mail bridge.
///
/// # Errors
///
/// Returns an error when request serialization, bridge execution, or response validation fails.
pub fn run(command: &Command, bridge: &mut impl MailBridge) -> Result<Value, ApplicationError> {
    let request = MailRequest::from(command);
    let request_json = serde_json::to_string(&request).map_err(ApplicationError::Serialize)?;
    let response_json = bridge
        .execute(&request_json)
        .map_err(ApplicationError::Bridge)?;

    let response =
        serde_json::from_str(&response_json).map_err(ApplicationError::InvalidResponse)?;
    if validate_response(&response)? {
        return Ok(response);
    }
    Err(ApplicationError::Mail(response))
}

fn validate_response(response: &Value) -> Result<bool, ApplicationError> {
    let object = response
        .as_object()
        .ok_or(ApplicationError::InvalidEnvelope(
            "response envelope must be an object",
        ))?;
    let is_ok =
        object
            .get("ok")
            .and_then(Value::as_bool)
            .ok_or(ApplicationError::InvalidEnvelope(
                "response envelope must contain a boolean ok field",
            ))?;

    let required_field = if is_ok { "data" } else { "error" };
    if object.contains_key(required_field) {
        return Ok(is_ok);
    }

    Err(ApplicationError::InvalidEnvelope(if is_ok {
        "successful response envelope must contain data"
    } else {
        "failed response envelope must contain error"
    }))
}

#[derive(Debug)]
pub enum ApplicationError {
    Serialize(serde_json::Error),
    Bridge(BridgeError),
    InvalidResponse(serde_json::Error),
    InvalidEnvelope(&'static str),
    Mail(Value),
}

impl ApplicationError {
    #[must_use]
    pub const fn response(&self) -> Option<&Value> {
        match self {
            Self::Mail(response) => Some(response),
            _ => None,
        }
    }

    #[must_use]
    pub fn exit_code(&self) -> u8 {
        match self {
            Self::Bridge(BridgeError::Timeout) => 4,
            Self::Mail(response) => match mail_error_code(response) {
                Some("invalid_request" | "unsupported_action") => 2,
                Some("permission_denied") => 3,
                Some("mail_timeout") => 4,
                _ => 5,
            },
            _ => 5,
        }
    }
}

impl Display for ApplicationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Serialize(error) => write!(formatter, "failed to serialize request: {error}"),
            Self::Bridge(error) => Display::fmt(error, formatter),
            Self::InvalidResponse(error) => {
                write!(formatter, "bridge returned invalid JSON: {error}")
            }
            Self::InvalidEnvelope(message) => {
                write!(formatter, "invalid response envelope: {message}")
            }
            Self::Mail(response) => formatter.write_str(
                response
                    .pointer("/error/message")
                    .and_then(Value::as_str)
                    .unwrap_or("Mail operation failed"),
            ),
        }
    }
}

impl Error for ApplicationError {}

fn mail_error_code(response: &Value) -> Option<&str> {
    response.pointer("/error/code").and_then(Value::as_str)
}
