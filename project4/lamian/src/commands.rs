use std::path::PathBuf;

use crate::cli::{Cli, Command};
use crate::db;
use crate::error::LamianError;
use crate::inject::{InjectRequest, inject_figure};

pub fn dispatch(cli: Cli) -> Result<(), LamianError> {
    match cli.command {
        Command::Init => {
            let vault_path = require_vault(cli.vault, "init")?;
            let paths = db::initialize_vault(&vault_path)?;
            println!("Initialized vault: {}", paths.vault_root.display());
            println!("Database path: {}", paths.database_path.display());
            Ok(())
        }
        Command::Inject {
            file_path,
            source_type,
            source_key,
            copy_mode,
        } => {
            let vault_path = require_vault(cli.vault, "inject")?;
            let result = inject_figure(InjectRequest {
                vault_root: vault_path,
                file_path,
                source_type,
                source_key,
                copy_mode,
            })?;

            println!("Injected figure: {}", result.figure_id);
            Ok(())
        }
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
