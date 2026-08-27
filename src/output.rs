use serde_json::{Value, json};

use crate::application::ApplicationError;

#[must_use]
pub fn error_envelope(error: &ApplicationError) -> Value {
    error.response().cloned().unwrap_or_else(|| {
        json!({
            "ok": false,
            "error": {
                "code": "bridge_error",
                "message": error.to_string(),
            },
        })
    })
}
