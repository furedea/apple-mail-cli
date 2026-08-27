use apple_mail_cli::application::run;
use apple_mail_cli::bridge::{BridgeError, MailBridge};
use apple_mail_cli::cli::Cli;
use clap::Parser;
use serde_json::{Value, json};

#[test]
fn application_sends_the_command_as_json_and_returns_the_bridge_response() {
    let cli =
        Cli::try_parse_from(["apple-mail", "accounts"]).expect("accounts command should parse");
    let mut bridge = FakeBridge::returning(r#"{"ok":true,"data":[]}"#);

    let response = run(cli.command(), &mut bridge).expect("application should run");

    assert_eq!(bridge.request(), Some(&json!({"action": "accounts"})));
    assert_eq!(response, json!({"ok": true, "data": []}));
}

#[test]
fn application_rejects_a_bridge_response_without_an_envelope() {
    let cli =
        Cli::try_parse_from(["apple-mail", "accounts"]).expect("accounts command should parse");
    let mut bridge = FakeBridge::returning("[]");

    let error = run(cli.command(), &mut bridge)
        .expect_err("a response without an envelope must be rejected");

    assert!(error.to_string().contains("response envelope"));
}

#[test]
fn application_returns_a_failed_bridge_envelope_as_an_error() {
    let cli =
        Cli::try_parse_from(["apple-mail", "accounts"]).expect("accounts command should parse");
    let mut bridge = FakeBridge::returning(
        r#"{"ok":false,"error":{"code":"permission_denied","message":"Allow Automation"}}"#,
    );

    let error = run(cli.command(), &mut bridge)
        .expect_err("a failed bridge envelope must produce an application error");

    assert_eq!(
        error.response(),
        Some(&json!({
            "ok": false,
            "error": {"code": "permission_denied", "message": "Allow Automation"},
        })),
    );
    assert_eq!(error.exit_code(), 3);
}

struct FakeBridge {
    response: String,
    request: Option<Value>,
}

impl FakeBridge {
    fn returning(response: &str) -> Self {
        Self {
            response: response.to_owned(),
            request: None,
        }
    }

    fn request(&self) -> Option<&Value> {
        self.request.as_ref()
    }
}

impl MailBridge for FakeBridge {
    fn execute(&mut self, request: &str) -> Result<String, BridgeError> {
        self.request = Some(serde_json::from_str(request).expect("request should be valid JSON"));
        Ok(self.response.clone())
    }
}
