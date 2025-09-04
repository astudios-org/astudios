mod cli;
mod commands;

use astudios::error::AstudiosError;
use clap::Parser;
use commands::CommandHandler;

fn main() {
    let cli = cli::Cli::parse();

    if let Err(e) = CommandHandler::handle(cli) {
        match e {
            AstudiosError::VersionNotFound(msg) => {
                eprintln!("Error: {msg}");
                eprintln!("Use 'astudios list' to see available versions");
            }
            AstudiosError::Download(msg) => {
                eprintln!("Download Error: {msg}");
            }
            AstudiosError::Installation(msg) => {
                eprintln!("Installation Error: {msg}");
            }
            AstudiosError::Extraction(msg) => {
                eprintln!("Extraction Error: {msg}");
            }
            AstudiosError::Network(e) => {
                eprintln!("Network Error: {e}");
                eprintln!("Please check your internet connection");
            }
            AstudiosError::Config(msg) => {
                eprintln!("Configuration Error: {msg}");
            }
            AstudiosError::PrerequisiteNotMet(msg) => {
                eprintln!("Prerequisite Check Failed: {msg}");
                eprintln!("Please resolve the above issues and try again");
            }
            AstudiosError::InsufficientResources(msg) => {
                eprintln!("Insufficient Resources: {msg}");
                eprintln!("Please free up space or resources and try again");
            }
            AstudiosError::PermissionDenied(msg) => {
                eprintln!("Permission Error: {msg}");
                eprintln!("You may need to run with administrator privileges");
            }
            AstudiosError::NetworkUnavailable(msg) => {
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
