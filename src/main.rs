#![forbid(unsafe_code)]

mod agents;
mod cli;
mod config;
mod detect;
mod display;
mod first_boot;
mod guest;
mod gui;
mod host;
mod notify;
mod process;
mod setup;
mod vm;

use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    let cli::Cli { command } = cli::Cli::parse();
    let Some(action) = command else {
        return if gui::run() {
            ExitCode::SUCCESS
        } else {
            ExitCode::FAILURE
        };
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
