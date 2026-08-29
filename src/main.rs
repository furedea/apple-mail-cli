use std::process::ExitCode;

use apple_mail_cli::application::run;
use apple_mail_cli::cli::Cli;
use apple_mail_cli::osascript::OsascriptBridge;
use apple_mail_cli::output::error_envelope;
use clap::Parser;

fn main() -> ExitCode {
    let cli = Cli::parse();
    let mut bridge = OsascriptBridge::new();

    match run(cli.command(), &mut bridge) {
        Ok(response) => {
            println!("{response}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{}", error_envelope(&error));
            ExitCode::from(error.exit_code())
        }
    }
}
