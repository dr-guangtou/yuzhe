mod cli;
mod commands;
mod db;
mod error;
mod inject;
mod link;
mod search;
mod tag;
mod update;

use clap::Parser;
use cli::Cli;
use error::LamianError;

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), LamianError> {
    let cli = Cli::parse();
    commands::dispatch(cli)
}
