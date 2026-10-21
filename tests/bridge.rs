use apple_mail_cli::bridge::BridgeError;

#[test]
fn failed_bridge_error_describes_the_exit_code_and_stderr() {
    let error = BridgeError::Failed {
        code: Some(1),
        stderr: "Mail got an error".to_owned(),
    };

    assert_eq!(
        error.to_string(),
        "bridge failed with exit code Some(1): Mail got an error"
    );
}

#[test]
fn invalid_output_bridge_error_describes_the_reason() {
    let error = BridgeError::InvalidOutput("stdout exceeded the supported limit".to_owned());

    assert_eq!(
        error.to_string(),
        "bridge returned invalid output: stdout exceeded the supported limit"
    );
}
