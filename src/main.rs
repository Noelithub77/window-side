#![forbid(unsafe_code)]

mod cli;
mod config;
mod detect;
mod display;
mod guest;
mod host;
mod notify;
mod process;
mod setup;
mod vm;

use clap::{CommandFactory, Parser};
use std::process::ExitCode;

fn main() -> ExitCode {
    let cli::Cli { command } = cli::Cli::parse();
    // The window arrives with the setup flow; until then bare `ws` shows help.
    let Some(action) = command else {
        let _ = cli::Cli::command().print_help();
        return ExitCode::SUCCESS;
    };
    let notifies = action.notifies();
    match cli::run(action) {
        Ok(code) => ExitCode::from(u8::try_from(code).unwrap_or(1)),
        Err(error) => {
            eprintln!("window-side: {error:#}");
            if notifies {
                notify::send(notify::Urgency::Critical, &format!("{error}"));
            }
            ExitCode::FAILURE
        }
    }
}
