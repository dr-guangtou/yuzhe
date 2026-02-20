use std::path::PathBuf;

use crate::cli::{Cli, Command, LinkAction, TagAction};
use crate::db;
use crate::error::LamianError;
use crate::inject::{InjectRequest, inject_figure};
use crate::link::{AddLinkRequest, RemoveLinkRequest, add_link, remove_link};
use crate::search::{SearchRequest, search_figures};
use crate::tag::{
    AddTagRequest, RemoveTagRequest, RenameTagRequest, add_tag_to_figure, remove_tag_from_figure,
    rename_tag,
};

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
        Command::Tag { action } => {
            let vault_path = require_vault(cli.vault, "tag")?;
            match action {
                TagAction::Add { figure_id, tag } => {
                    let result = add_tag_to_figure(AddTagRequest {
                        vault_root: vault_path,
                        figure_id,
                        tag,
                    })?;

                    if result.created_relation {
                        println!("Added tag: {}", result.normalized_tag);
                    } else {
                        println!("Tag already assigned: {}", result.normalized_tag);
                    }
                    Ok(())
                }
                TagAction::Remove { figure_id, tag } => {
                    let result = remove_tag_from_figure(RemoveTagRequest {
                        vault_root: vault_path,
                        figure_id,
                        tag,
                    })?;

                    if result.removed_relation {
                        println!("Removed tag: {}", result.normalized_tag);
                    } else {
                        println!("Tag not assigned: {}", result.normalized_tag);
                    }
                    Ok(())
                }
                TagAction::Rename { old_tag, new_tag } => {
                    let result = rename_tag(RenameTagRequest {
                        vault_root: vault_path,
                        old_tag,
                        new_tag,
                    })?;

                    if result.renamed_count == 0 {
                        println!("Tag unchanged: {}", result.normalized_old_tag);
                    } else {
                        println!(
                            "Renamed tag: {} -> {} (affected: {})",
                            result.normalized_old_tag,
                            result.normalized_new_tag,
                            result.renamed_count
                        );
                    }
                    Ok(())
                }
            }
        }
        Command::Link { action } => {
            let vault_path = require_vault(cli.vault, "link")?;
            match action {
                LinkAction::Add {
                    from_figure_id,
                    to_figure_id,
                    relation,
                } => {
                    let result = add_link(AddLinkRequest {
                        vault_root: vault_path,
                        from_figure_id,
                        to_figure_id,
                        relation,
                    })?;

                    if result.created_link {
                        println!(
                            "Added link: {} -> {} [{}]",
                            result.from_figure_id, result.to_figure_id, result.normalized_relation
                        );
                    } else {
                        println!(
                            "Link already exists: {} -> {} [{}]",
                            result.from_figure_id, result.to_figure_id, result.normalized_relation
                        );
                    }
                    Ok(())
                }
                LinkAction::Remove {
                    from_figure_id,
                    to_figure_id,
                } => {
                    let result = remove_link(RemoveLinkRequest {
                        vault_root: vault_path,
                        from_figure_id,
                        to_figure_id,
                    })?;

                    println!(
                        "Removed links: {} -> {} (count: {})",
                        result.from_figure_id, result.to_figure_id, result.removed_count
                    );
                    Ok(())
                }
            }
        }
        Command::Search {
            tag,
            source_key,
            text,
        } => {
            let vault_path = require_vault(cli.vault, "search")?;
            let result = search_figures(SearchRequest {
                vault_root: vault_path,
                tag,
                source_key,
                text,
            })?;

            println!("Search results: {}", result.figures.len());
            if result.figures.is_empty() {
                println!("No figures matched.");
            } else {
                for figure in result.figures {
                    println!("{} | {}", figure.figure_id, figure.display_name);
                }
            }
            Ok(())
        }
        Command::Export { .. } => Err(LamianError::NotImplemented { command: "export" }),
    }
}

fn require_vault(
    vault_argument: Option<PathBuf>,
    command: &'static str,
) -> Result<PathBuf, LamianError> {
    vault_argument.ok_or(LamianError::MissingVaultArgument { command })
}
