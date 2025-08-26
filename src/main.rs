mod api;
mod cli;
mod commands;
mod downloader;
mod installer;
mod model;

use clap::Parser;
use commands::CommandHandler;

fn main() {
    let cli = cli::Cli::parse();

    if let Err(e) = CommandHandler::handle(cli) {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
