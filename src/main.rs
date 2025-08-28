mod cli;
mod commands;

use as_man::error::AsManError;
use clap::Parser;
use commands::CommandHandler;

fn main() {
    let cli = cli::Cli::parse();

    if let Err(e) = CommandHandler::handle(cli) {
        match e {
            AsManError::VersionNotFound(msg) => {
                eprintln!("Error: {msg}");
                eprintln!("Use 'as-man list' to see available versions");
            }
            AsManError::Download(msg) => {
                eprintln!("Download Error: {msg}");
            }
            AsManError::Installation(msg) => {
                eprintln!("Installation Error: {msg}");
            }
            AsManError::Extraction(msg) => {
                eprintln!("Extraction Error: {msg}");
            }
            AsManError::Network(e) => {
                eprintln!("Network Error: {e}");
                eprintln!("Please check your internet connection");
            }
            AsManError::Config(msg) => {
                eprintln!("Configuration Error: {msg}");
            }
            AsManError::PrerequisiteNotMet(msg) => {
                eprintln!("Prerequisite Check Failed: {msg}");
                eprintln!("Please resolve the above issues and try again");
            }
            AsManError::InsufficientResources(msg) => {
                eprintln!("Insufficient Resources: {msg}");
                eprintln!("Please free up space or resources and try again");
            }
            AsManError::PermissionDenied(msg) => {
                eprintln!("Permission Error: {msg}");
                eprintln!("You may need to run with administrator privileges");
            }
            AsManError::NetworkUnavailable(msg) => {
                eprintln!("Network Error: {msg}");
                eprintln!("Please check your internet connection and firewall settings");
            }
            _ => {
                eprintln!("Error: {e}");
            }
        }
        std::process::exit(1);
    }
}
