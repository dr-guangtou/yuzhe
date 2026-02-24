use clap::Parser;
use lamian::cli::Cli;
use lamian::commands;
use lamian::error::LamianError;

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
