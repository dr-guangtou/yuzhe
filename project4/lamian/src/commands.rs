use std::path::PathBuf;

use serde::Serialize;
use serde_json::json;

use crate::bundle::{
    bundle_export, bundle_import, bundle_inspect, BundleExportRequest, BundleImportRequest,
    BundleInspectRequest,
};
use crate::cli::{
    BundleAction, Cli, CollectionAction, Command, LinkAction, QueryAction, SourceAction, TagAction,
};
use crate::collection::{
    add_collection_item, create_collection, delete_collection, list_collections,
    remove_collection_item, update_collection, AddCollectionItemRequest, CreateCollectionRequest,
    DeleteCollectionRequest, ListCollectionsRequest, RemoveCollectionItemRequest,
    UpdateCollectionRequest,
};
use crate::db;
use crate::delete::{delete_figure, DeleteFigureRequest};
use crate::doctor::{doctor_vault, DoctorRequest};
use crate::error::LamianError;
use crate::export::{export_metadata, ExportRequest};
use crate::import::{import_batch, ImportRequest};
use crate::inject::{inject_figure, InjectRequest};
use crate::link::{add_link, remove_link, AddLinkRequest, RemoveLinkRequest};
use crate::list::{list_figures, ListFiguresRequest};
use crate::open::{open_figure, OpenFigureRequest};
use crate::query::{
    delete_query, list_queries, run_query, save_query, DeleteQueryRequest, ListQueriesRequest,
    RunQueryRequest, SaveQueryRequest,
};
use crate::search::{search_figures, SearchRequest};
use crate::show::{show_figure, ShowFigureRequest};
use crate::source::{update_source_metadata, UpdateSourceRequest};
use crate::tag::{
    add_tag_to_figure, list_tags, remove_tag_from_figure, rename_tag, AddTagRequest,
    ListTagsRequest, RemoveTagRequest, RenameTagRequest,
};
use crate::update::{update_figure, UpdateRequest};
use crate::verify::{verify_vault, VerifyRequest};

pub fn dispatch(cli: Cli) -> Result<(), LamianError> {
    let Cli {
        vault,
        json_output,
        command,
    } = cli;

    match command {
        Command::Init => {
            let vault_path = require_vault(vault, "init")?;
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
            let vault_path = require_vault(vault, "inject")?;
            let result = inject_figure(InjectRequest {
                vault_root: vault_path,
                file_path,
                source_type,
                source_key,
                copy_mode,
            })?;

            if json_output {
                print_json(&json!({
                    "command": "inject",
                    "status": "ok",
                    "result": result,
                }))?;
            } else {
                println!("Injected figure: {}", result.figure_id);
            }
            Ok(())
        }
        Command::Update {
            figure_id,
            name,
            caption,
            clear_caption,
            note_file,
        } => {
            let vault_path = require_vault(vault, "update")?;
            let result = update_figure(UpdateRequest {
                vault_root: vault_path,
                figure_id,
                name,
                caption,
                clear_caption,
                note_file,
            })?;

            if json_output {
                print_json(&json!({
                    "command": "update",
                    "status": "ok",
                    "result": result,
                }))?;
            } else {
                println!("Updated figure: {}", result.figure_id);
                println!("Updated fields: {}", result.updated_fields.join(", "));
            }
            Ok(())
        }
        Command::Tag { action } => {
            let vault_path = require_vault(vault, "tag")?;
            match action {
                TagAction::Add { figure_id, tag } => {
                    let result = add_tag_to_figure(AddTagRequest {
                        vault_root: vault_path,
                        figure_id,
                        tag,
                    })?;

                    if json_output {
                        print_json(&json!({
                            "command": "tag.add",
                            "status": "ok",
                            "result": result,
                        }))?;
                    } else if result.created_relation {
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

                    if json_output {
                        print_json(&json!({
                            "command": "tag.remove",
                            "status": "ok",
                            "result": result,
                        }))?;
                    } else if result.removed_relation {
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

                    if json_output {
                        print_json(&json!({
                            "command": "tag.rename",
                            "status": "ok",
                            "result": result,
                        }))?;
                    } else if result.renamed_count == 0 {
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
                TagAction::List => {
                    let result = list_tags(ListTagsRequest {
                        vault_root: vault_path,
                    })?;

                    if json_output {
                        print_json(&json!({
                            "command": "tag.list",
                            "status": "ok",
                            "count": result.tags.len(),
                            "tags": result.tags,
                        }))?;
                    } else {
                        println!("Tags: {}", result.tags.len());
                        if result.tags.is_empty() {
                            println!("No tags found.");
                        } else {
                            for tag in result.tags {
                                println!("{} | figures={}", tag.tag_name, tag.figure_count);
                            }
                        }
                    }
                    Ok(())
                }
            }
        }
        Command::Source { action } => {
            let vault_path = require_vault(vault, "source")?;
            match action {
                SourceAction::Update {
                    figure_id,
                    title,
                    authors,
                    published_at,
                    clear_title,
                    clear_authors,
                    clear_published_at,
                } => {
                    let result = update_source_metadata(UpdateSourceRequest {
                        vault_root: vault_path,
                        figure_id,
                        title,
                        authors,
                        published_at,
                        clear_title,
                        clear_authors,
                        clear_published_at,
                    })?;

                    println!("Updated source metadata for figure: {}", result.figure_id);
                    println!("Updated fields: {}", result.updated_fields.join(", "));
                    Ok(())
                }
            }
        }
        Command::Link { action } => {
            let vault_path = require_vault(vault, "link")?;
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

                    if json_output {
                        print_json(&json!({
                            "command": "link.add",
                            "status": "ok",
                            "result": result,
                        }))?;
                    } else if result.created_link {
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

                    if json_output {
                        print_json(&json!({
                            "command": "link.remove",
                            "status": "ok",
                            "result": result,
                        }))?;
                    } else {
                        println!(
                            "Removed links: {} -> {} (count: {})",
                            result.from_figure_id, result.to_figure_id, result.removed_count
                        );
                    }
                    Ok(())
                }
            }
        }
        Command::Search {
            tag,
            tag_prefix,
            source_key,
            text,
        } => {
            let vault_path = require_vault(vault, "search")?;
            let result = search_figures(SearchRequest {
                vault_root: vault_path,
                tag,
                tag_prefix,
                source_key,
                text,
            })?;

            if json_output {
                print_json(&json!({
                    "command": "search",
                    "status": "ok",
                    "count": result.figures.len(),
                    "result": result,
                }))?;
            } else {
                println!("Search results: {}", result.figures.len());
                if result.figures.is_empty() {
                    println!("No figures matched.");
                } else {
                    for figure in result.figures {
                        println!("{} | {}", figure.figure_id, figure.display_name);
                    }
                }
            }
            Ok(())
        }
        Command::List { sort, order, limit } => {
            let vault_path = require_vault(vault, "list")?;
            let result = list_figures(ListFiguresRequest {
                vault_root: vault_path,
                sort,
                order,
                limit,
            })?;

            println!("List results: {}", result.figures.len());
            if result.figures.is_empty() {
                println!("No figures found.");
            } else {
                for figure in result.figures {
                    println!(
                        "{} | {} | created_at={} | updated_at={}",
                        figure.figure_id, figure.display_name, figure.created_at, figure.updated_at
                    );
                }
            }
            Ok(())
        }
        Command::Show { figure_id } => {
            let vault_path = require_vault(vault, "show")?;
            let result = show_figure(ShowFigureRequest {
                vault_root: vault_path,
                figure_id,
            })?;

            println!("Figure: {}", result.figure_id);
            println!("Display name: {}", result.display_name);
            println!(
                "Caption: {}",
                optional_text_or_none(result.caption.as_deref())
            );
            println!("File path: {}", result.file_path);
            println!("File hash sha256: {}", result.file_hash_sha256);
            println!("Media type: {}", result.media_type);
            println!("File size bytes: {}", result.file_size_bytes);
            println!("Created at: {}", result.created_at);
            println!("Updated at: {}", result.updated_at);

            println!("Sources ({}):", result.sources.len());
            for source in result.sources {
                println!(
                    "- {} | {} | title={} | authors={} | published_at={} | created_at={}",
                    source.source_type,
                    source.source_key,
                    optional_text_or_none(source.source_title.as_deref()),
                    optional_text_or_none(source.source_authors.as_deref()),
                    optional_text_or_none(source.source_published_at.as_deref()),
                    source.created_at
                );
            }

            println!("Tags ({}):", result.tags.len());
            for tag in result.tags {
                println!("- {tag}");
            }

            println!("Outbound links ({}):", result.outbound_links.len());
            for link in result.outbound_links {
                println!(
                    "- {} | relation={} | created_at={}",
                    link.to_figure_id, link.relation_type, link.created_at
                );
            }

            match result.note {
                Some(note) => {
                    println!("Note updated at: {}", note.updated_at);
                    println!("Note markdown: {}", note.note_markdown);
                }
                None => {
                    println!("Note: (none)");
                }
            }
            Ok(())
        }
        Command::Open { figure_id } => {
            let vault_path = require_vault(vault, "open")?;
            let result = open_figure(OpenFigureRequest {
                vault_root: vault_path,
                figure_id,
            })?;

            println!("Opened figure: {}", result.figure_id);
            println!("Opened path: {}", result.resolved_file_path.display());
            Ok(())
        }
        Command::Delete { figure_id } => {
            let vault_path = require_vault(vault, "delete")?;
            let result = delete_figure(DeleteFigureRequest {
                vault_root: vault_path,
                figure_id,
            })?;

            println!("Deleted figure: {}", result.figure_id);
            println!(
                "Removed managed file: {}",
                yes_no(result.removed_managed_file)
            );
            println!("Removed orphan tags: {}", result.removed_orphan_tag_count);
            Ok(())
        }
        Command::Query { action } => {
            let vault_path = require_vault(vault, "query")?;
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
                QueryAction::Run {
                    name_or_id,
                    detail,
                    reference_mode,
                } => {
                    let result = run_query(RunQueryRequest {
                        vault_root: vault_path,
                        query_reference: name_or_id,
                        reference_mode,
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
                QueryAction::Delete {
                    name_or_id,
                    reference_mode,
                } => {
                    let result = delete_query(DeleteQueryRequest {
                        vault_root: vault_path,
                        query_reference: name_or_id,
                        reference_mode,
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
            let vault_path = require_vault(vault, "collection")?;
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
                    reference_mode,
                } => {
                    let result = add_collection_item(AddCollectionItemRequest {
                        vault_root: vault_path,
                        collection_reference: collection,
                        reference_mode,
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
                    reference_mode,
                } => {
                    let result = remove_collection_item(RemoveCollectionItemRequest {
                        vault_root: vault_path,
                        collection_reference: collection,
                        reference_mode,
                        figure_id,
                    })?;

                    print_json(&json!({
                        "command": "collection.remove",
                        "status": "ok",
                        "result": result,
                    }))?;
                    Ok(())
                }
                CollectionAction::List {
                    collection,
                    reference_mode,
                } => {
                    let result = list_collections(ListCollectionsRequest {
                        vault_root: vault_path,
                        collection_reference: collection,
                        reference_mode,
                    })?;

                    print_json(&json!({
                        "command": "collection.list",
                        "status": "ok",
                        "count": result.collections.len(),
                        "collections": result.collections,
                    }))?;
                    Ok(())
                }
                CollectionAction::Delete {
                    collection,
                    reference_mode,
                } => {
                    let result = delete_collection(DeleteCollectionRequest {
                        vault_root: vault_path,
                        collection_reference: collection,
                        reference_mode,
                    })?;

                    print_json(&json!({
                        "command": "collection.delete",
                        "status": "ok",
                        "result": result,
                    }))?;
                    Ok(())
                }
                CollectionAction::Update {
                    collection,
                    reference_mode,
                    name,
                    query_id,
                    clear_query_id,
                } => {
                    let result = update_collection(UpdateCollectionRequest {
                        vault_root: vault_path,
                        collection_reference: collection,
                        reference_mode,
                        name,
                        query_id,
                        clear_query_id,
                    })?;

                    print_json(&json!({
                        "command": "collection.update",
                        "status": "ok",
                        "result": result,
                    }))?;
                    Ok(())
                }
            }
        }
        Command::Bundle { action } => match action {
            BundleAction::Export { target } => {
                let vault_path = require_vault(vault, "bundle export")?;
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
            BundleAction::Inspect { bundle_path } => {
                let result = bundle_inspect(BundleInspectRequest { bundle_path })?;

                print_json(&json!({
                    "command": "bundle.inspect",
                    "status": "ok",
                    "result": result,
                }))?;
                Ok(())
            }
            BundleAction::Import {
                bundle_path,
                fail_on_link_loss,
                dry_run,
                on_conflict,
            } => {
                let vault_path = require_vault(vault, "bundle import")?;
                let result = bundle_import(BundleImportRequest {
                    vault_root: vault_path,
                    bundle_path,
                    fail_on_link_loss,
                    dry_run,
                    on_conflict,
                })?;

                print_json(&json!({
                    "command": "bundle.import",
                    "status": "ok",
                    "result": result,
                }))?;
                Ok(())
            }
        },
        Command::Import {
            input_path,
            source_type,
            source_key_template,
            copy_mode,
            recursive,
            dry_run,
        } => {
            let vault_path = require_vault(vault, "import")?;
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
            let vault_path = require_vault(vault, "doctor")?;
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
        Command::Verify => {
            let vault_path = require_vault(vault, "verify")?;
            let result = verify_vault(VerifyRequest {
                vault_root: vault_path,
            })?;

            let status = if result.issue_count == 0 {
                "ok"
            } else {
                "issues_found"
            };
            let issue_count = result.issue_count;
            print_json(&json!({
                "command": "verify",
                "status": status,
                "result": result,
            }))?;

            if issue_count > 0 {
                return Err(LamianError::VerifyIssuesFound { issue_count });
            }

            Ok(())
        }
        Command::Export { format, target } => {
            let vault_path = require_vault(vault, "export")?;
            let result = export_metadata(ExportRequest {
                vault_root: vault_path,
                format,
                target,
            })?;

            if json_output {
                print_json(&json!({
                    "command": "export",
                    "status": "ok",
                    "result": result,
                }))?;
            } else if let Some(path) = result.target_path {
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

fn optional_text_or_none(value: Option<&str>) -> &str {
    value.unwrap_or("(none)")
}

fn yes_no(value: bool) -> &'static str {
    if value {
        "yes"
    } else {
        "no"
    }
}
