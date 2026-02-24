mod bundle;
mod cli;
mod collection;
mod commands;
mod db;
mod delete;
mod doctor;
mod error;
mod export;
mod import;
mod inject;
mod link;
mod list;
mod open;
mod query;
mod search;
mod show;
mod source;
mod tag;
mod tag_validation;
mod update;
mod verify;

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
