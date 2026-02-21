use std::path::PathBuf;

use serde::Serialize;
use serde_json::json;

use crate::bundle::{bundle_export, bundle_import, BundleExportRequest, BundleImportRequest};
use crate::cli::{
    BundleAction, Cli, CollectionAction, Command, LinkAction, QueryAction, TagAction,
};
use crate::collection::{
    add_collection_item, create_collection, delete_collection, list_collections,
    remove_collection_item, AddCollectionItemRequest, CreateCollectionRequest,
    DeleteCollectionRequest, ListCollectionsRequest, RemoveCollectionItemRequest,
};
use crate::db;
use crate::doctor::{doctor_vault, DoctorRequest};
use crate::error::LamianError;
use crate::export::{export_metadata, ExportRequest};
use crate::import::{import_batch, ImportRequest};
use crate::inject::{inject_figure, InjectRequest};
use crate::link::{add_link, remove_link, AddLinkRequest, RemoveLinkRequest};
use crate::query::{
    delete_query, list_queries, run_query, save_query, DeleteQueryRequest, ListQueriesRequest,
    RunQueryRequest, SaveQueryRequest,
};
use crate::search::{search_figures, SearchRequest};
use crate::tag::{
    add_tag_to_figure, remove_tag_from_figure, rename_tag, AddTagRequest, RemoveTagRequest,
    RenameTagRequest,
};
use crate::update::{update_figure, UpdateRequest};

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
        Command::Update {
            figure_id,
            name,
            caption,
            note_file,
        } => {
            let vault_path = require_vault(cli.vault, "update")?;
            let result = update_figure(UpdateRequest {
                vault_root: vault_path,
                figure_id,
                name,
                caption,
                note_file,
            })?;

            println!("Updated figure: {}", result.figure_id);
            println!("Updated fields: {}", result.updated_fields.join(", "));
            Ok(())
        }
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
        Command::Query { action } => {
            let vault_path = require_vault(cli.vault, "query")?;
            match action {
                QueryAction::Save {
                    name,
                    tag,
                    source_key,
                    text,
                    sort,
                    order,
                    limit,
                } => {
                    let result = save_query(SaveQueryRequest {
                        vault_root: vault_path,
                        query_name: name,
                        tag,
                        source_key,
                        text,
                        sort_field: sort,
                        sort_order: order,
                        limit,
                    })?;

                    print_json(&json!({
                        "command": "query.save",
                        "status": "ok",
                        "result": result,
                    }))?;
                    Ok(())
                }
                QueryAction::Run { name_or_id, detail } => {
                    let result = run_query(RunQueryRequest {
                        vault_root: vault_path,
                        query_reference: name_or_id,
                        detail,
                    })?;

                    print_json(&json!({
                        "command": "query.run",
                        "status": "ok",
                        "result": result,
                    }))?;
                    Ok(())
                }
                QueryAction::List => {
                    let result = list_queries(ListQueriesRequest {
                        vault_root: vault_path,
                    })?;

                    print_json(&json!({
                        "command": "query.list",
                        "status": "ok",
                        "count": result.queries.len(),
                        "queries": result.queries,
                    }))?;
                    Ok(())
                }
                QueryAction::Delete { name_or_id } => {
                    let result = delete_query(DeleteQueryRequest {
                        vault_root: vault_path,
                        query_reference: name_or_id,
                    })?;

                    print_json(&json!({
                        "command": "query.delete",
                        "status": "ok",
                        "result": result,
                    }))?;
                    Ok(())
                }
            }
        }
        Command::Collection { action } => {
            let vault_path = require_vault(cli.vault, "collection")?;
            match action {
                CollectionAction::Create { name, query_id } => {
                    let result = create_collection(CreateCollectionRequest {
                        vault_root: vault_path,
                        collection_name: name,
                        query_id,
                    })?;

                    print_json(&json!({
                        "command": "collection.create",
                        "status": "ok",
                        "result": result,
                    }))?;
                    Ok(())
                }
                CollectionAction::Add {
                    collection,
                    figure_id,
                } => {
                    let result = add_collection_item(AddCollectionItemRequest {
                        vault_root: vault_path,
                        collection_reference: collection,
                        figure_id,
                    })?;

                    print_json(&json!({
                        "command": "collection.add",
                        "status": "ok",
                        "result": result,
                    }))?;
                    Ok(())
                }
                CollectionAction::Remove {
                    collection,
                    figure_id,
                } => {
                    let result = remove_collection_item(RemoveCollectionItemRequest {
                        vault_root: vault_path,
                        collection_reference: collection,
                        figure_id,
                    })?;

                    print_json(&json!({
                        "command": "collection.remove",
                        "status": "ok",
                        "result": result,
                    }))?;
                    Ok(())
                }
                CollectionAction::List { collection } => {
                    let result = list_collections(ListCollectionsRequest {
                        vault_root: vault_path,
                        collection_reference: collection,
                    })?;

                    print_json(&json!({
                        "command": "collection.list",
                        "status": "ok",
                        "count": result.collections.len(),
                        "collections": result.collections,
                    }))?;
                    Ok(())
                }
                CollectionAction::Delete { collection } => {
                    let result = delete_collection(DeleteCollectionRequest {
                        vault_root: vault_path,
                        collection_reference: collection,
                    })?;

                    print_json(&json!({
                        "command": "collection.delete",
                        "status": "ok",
                        "result": result,
                    }))?;
                    Ok(())
                }
            }
        }
        Command::Bundle { action } => {
            let vault_path = require_vault(cli.vault, "bundle")?;
            match action {
                BundleAction::Export { target } => {
                    let result = bundle_export(BundleExportRequest {
                        vault_root: vault_path,
                        target_path: target,
                    })?;

                    print_json(&json!({
                        "command": "bundle.export",
                        "status": "ok",
                        "result": result,
                    }))?;
                    Ok(())
                }
                BundleAction::Import { bundle_path } => {
                    let result = bundle_import(BundleImportRequest {
                        vault_root: vault_path,
                        bundle_path,
                    })?;

                    print_json(&json!({
                        "command": "bundle.import",
                        "status": "ok",
                        "result": result,
                    }))?;
                    Ok(())
                }
            }
        }
        Command::Import {
            input_path,
            source_type,
            source_key_template,
            copy_mode,
            recursive,
            dry_run,
        } => {
            let vault_path = require_vault(cli.vault, "import")?;
            let result = import_batch(ImportRequest {
                vault_root: vault_path,
                input_path,
                source_type,
                source_key_template,
                copy_mode,
                recursive,
                dry_run,
            })?;

            let status = if result.has_failures() {
                "partial_failure"
            } else {
                "ok"
            };
            let failed_count = result.failed;
            print_json(&json!({
                "command": "import",
                "status": status,
                "result": result,
            }))?;

            if failed_count > 0 {
                return Err(LamianError::ImportCompletedWithFailures { failed_count });
            }

            Ok(())
        }
        Command::Doctor { fix } => {
            let vault_path = require_vault(cli.vault, "doctor")?;
            let result = doctor_vault(DoctorRequest {
                vault_root: vault_path,
                fix,
            })?;

            let status = if result.unresolved_count == 0 {
                "ok"
            } else {
                "issues_found"
            };
            let unresolved_count = result.unresolved_count;
            print_json(&json!({
                "command": "doctor",
                "status": status,
                "result": result,
            }))?;

            if unresolved_count > 0 {
                return Err(LamianError::DoctorIssuesFound {
                    issue_count: unresolved_count,
                });
            }

            Ok(())
        }
        Command::Export { format, target } => {
            let vault_path = require_vault(cli.vault, "export")?;
            let result = export_metadata(ExportRequest {
                vault_root: vault_path,
                format,
                target,
            })?;

            if let Some(path) = result.target_path {
                println!(
                    "Exported metadata: {} figures -> {}",
                    result.figure_count,
                    path.display()
                );
            } else if let Some(output) = result.output {
                print!("{output}");
            }
            Ok(())
        }
    }
}

fn require_vault(
    vault_argument: Option<PathBuf>,
    command: &'static str,
) -> Result<PathBuf, LamianError> {
    vault_argument.ok_or(LamianError::MissingVaultArgument { command })
}

fn print_json<T: Serialize>(value: &T) -> Result<(), LamianError> {
    let content = serde_json::to_string_pretty(value).map_err(|error| {
        LamianError::JsonOutputSerializationFailed {
            reason: error.to_string(),
        }
    })?;
    println!("{content}");
    Ok(())
}
