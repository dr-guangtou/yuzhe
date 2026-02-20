use std::path::PathBuf;

use crate::cli::{Cli, Command};
use crate::db;
use crate::error::LamianError;

pub fn dispatch(cli: Cli) -> Result<(), LamianError> {
    match cli.command {
        Command::Init => {
            let vault_path = require_vault(cli.vault, "init")?;
            let paths = db::initialize_vault(&vault_path)?;
            println!("Initialized vault: {}", paths.vault_root.display());
            println!("Database path: {}", paths.database_path.display());
            Ok(())
        }
        Command::Inject { .. } => Err(LamianError::NotImplemented { command: "inject" }),
        Command::Update { .. } => Err(LamianError::NotImplemented { command: "update" }),
        Command::Tag { .. } => Err(LamianError::NotImplemented { command: "tag" }),
        Command::Link { .. } => Err(LamianError::NotImplemented { command: "link" }),
        Command::Search { .. } => Err(LamianError::NotImplemented { command: "search" }),
        Command::Export { .. } => Err(LamianError::NotImplemented { command: "export" }),
    }
}

fn require_vault(
    vault_argument: Option<PathBuf>,
    command: &'static str,
) -> Result<PathBuf, LamianError> {
    vault_argument.ok_or(LamianError::MissingVaultArgument { command })
}
