use std::path::{Path, PathBuf};

use eframe::egui;

use crate::cli::{ListSortField, ListSortOrder};
use crate::delete::{delete_figure, DeleteFigureRequest};
use crate::inject::{inject_figure, CopyMode, InjectRequest, SourceType};
use crate::link::{add_link, remove_link, AddLinkRequest, RemoveLinkRequest};
use crate::list::{list_figures, ListFigureRow, ListFiguresRequest};
use crate::search::{search_figures, SearchFigure, SearchRequest};
use crate::show::{show_figure, ShowFigureRequest, ShowFigureResult};
use crate::source::{update_source_metadata, UpdateSourceRequest};
use crate::tag::{add_tag_to_figure, remove_tag_from_figure, AddTagRequest, RemoveTagRequest};
use crate::update::{update_figure, UpdateRequest};

#[derive(Debug, Clone)]
struct FigureListRowView {
    figure_id: String,
    display_name: String,
    created_at: Option<String>,
    updated_at: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FigureEditorLifecycle {
    EditingClean,
    EditingDirty,
    Saving,
    SaveFailed,
}

#[derive(Debug, Clone)]
struct FigureMetadataDraft {
    figure_id: String,
    original_display_name: String,
    original_caption: Option<String>,
    display_name_input: String,
    caption_input: String,
    clear_caption: bool,
    lifecycle: FigureEditorLifecycle,
    last_error: Option<String>,
}

#[derive(Debug, Clone)]
struct FigureUpdatePayload {
    figure_id: String,
    name: Option<String>,
    caption: Option<String>,
    clear_caption: bool,
    has_changes: bool,
    display_name_changed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SourceEditorLifecycle {
    EditingClean,
    EditingDirty,
    Saving,
    SaveFailed,
}

#[derive(Debug, Clone)]
struct SourceMetadataDraft {
    figure_id: String,
    original_title: Option<String>,
    original_authors: Option<String>,
    original_published_at: Option<String>,
    title_input: String,
    authors_input: String,
    published_at_input: String,
    clear_title: bool,
    clear_authors: bool,
    clear_published_at: bool,
    lifecycle: SourceEditorLifecycle,
    last_error: Option<String>,
}

#[derive(Debug, Clone)]
struct SourceUpdatePayload {
    figure_id: String,
    title: Option<String>,
    authors: Option<String>,
    published_at: Option<String>,
    clear_title: bool,
    clear_authors: bool,
    clear_published_at: bool,
    has_changes: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TagEditorLifecycle {
    EditingClean,
    EditingDirty,
    Saving,
    SaveFailed,
}

#[derive(Debug, Clone)]
struct TagMutationDraft {
    figure_id: String,
    tag_input: String,
    lifecycle: TagEditorLifecycle,
    last_error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TagMutationAction {
    Add,
    Remove,
}

#[derive(Debug, Clone)]
struct TagMutationPayload {
    figure_id: String,
    tag: String,
    action: TagMutationAction,
    has_changes: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LinkEditorLifecycle {
    EditingClean,
    EditingDirty,
    Saving,
    SaveFailed,
}

#[derive(Debug, Clone)]
struct LinkMutationDraft {
    figure_id: String,
    to_figure_id_input: String,
    relation_input: String,
    lifecycle: LinkEditorLifecycle,
    last_error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LinkMutationAction {
    Add,
    Remove,
}

#[derive(Debug, Clone)]
struct LinkMutationPayload {
    figure_id: String,
    to_figure_id: String,
    relation: String,
    action: LinkMutationAction,
    has_changes: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DeleteEditorLifecycle {
    ConfirmingDelete,
    Deleting,
    DeleteFailed,
}

#[derive(Debug, Clone)]
struct DeleteFigureDraft {
    figure_id: String,
    lifecycle: DeleteEditorLifecycle,
    last_error: Option<String>,
}

#[derive(Debug, Clone)]
struct DeleteFigurePayload {
    figure_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum DropIngestLifecycle {
    #[default]
    Idle,
    DropReceived,
    MetadataRequired,
    ReadyToCommit,
    Committing,
    Committed,
    CommitFailed,
}

#[derive(Debug, Clone, Default)]
struct DropIngestItemDraft {
    input_path: PathBuf,
    normalized_path: String,
    source_type_input: String,
    source_key_input: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DropIngestItemCommitStatus {
    Imported,
    SkippedDuplicate,
    Failed,
}

#[derive(Debug, Clone)]
struct DropIngestItemCommitResult {
    normalized_path: String,
    status: DropIngestItemCommitStatus,
    figure_id: Option<String>,
    error: Option<String>,
}

#[derive(Debug, Clone, Default)]
struct DropIngestSessionDraft {
    lifecycle: DropIngestLifecycle,
    dropped_items: Vec<DropIngestItemDraft>,
    last_commit_results: Vec<DropIngestItemCommitResult>,
    last_error: Option<String>,
}

#[derive(Default)]
pub struct LamianGuiApp {
    vault_root_input: String,
    connected_vault_root: Option<PathBuf>,
    search_text: String,
    figure_rows: Vec<FigureListRowView>,
    selected_figure_id: Option<String>,
    selected_figure_detail: Option<ShowFigureResult>,
    figure_metadata_draft: Option<FigureMetadataDraft>,
    source_metadata_draft: Option<SourceMetadataDraft>,
    tag_mutation_draft: Option<TagMutationDraft>,
    link_mutation_draft: Option<LinkMutationDraft>,
    delete_figure_draft: Option<DeleteFigureDraft>,
    drop_ingest_session: DropIngestSessionDraft,
    status_message: Option<String>,
    error_message: Option<String>,
}

impl LamianGuiApp {
    pub fn new(_creation_context: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }

    fn connect_and_refresh(&mut self) {
        let vault_root_value = self.vault_root_input.trim();
        if vault_root_value.is_empty() {
            self.error_message = Some("Vault path is required.".to_string());
            return;
        }

        self.connected_vault_root = Some(PathBuf::from(vault_root_value));
        self.refresh_figure_rows();
    }

    fn refresh_figure_rows(&mut self) {
        let Some(vault_root) = self.connected_vault_root.as_ref() else {
            self.error_message = Some("Open a vault first.".to_string());
            return;
        };

        let search_value = self.search_text.trim();
        let row_result = if search_value.is_empty() {
            list_figures(ListFiguresRequest {
                vault_root: vault_root.clone(),
                sort: ListSortField::FigureId,
                order: ListSortOrder::Asc,
                limit: None,
            })
            .map(|result| figure_rows_from_list(result.figures))
        } else {
            search_figures(SearchRequest {
                vault_root: vault_root.clone(),
                tag: None,
                tag_prefix: None,
                source_key: None,
                text: Some(search_value.to_string()),
            })
            .map(|result| figure_rows_from_search(result.figures))
        };

        match row_result {
            Ok(rows) => {
                self.figure_rows = rows;
                self.status_message = Some(format!("Loaded {} figures.", self.figure_rows.len()));
                self.error_message = None;
                self.sync_selection_after_row_reload();
            }
            Err(error) => {
                self.error_message = Some(error.to_string());
                self.status_message = None;
            }
        }
    }

    fn sync_selection_after_row_reload(&mut self) {
        let Some(selected_figure_id) = self.selected_figure_id.clone() else {
            return;
        };

        if self
            .figure_rows
            .iter()
            .any(|row| row.figure_id == selected_figure_id)
        {
            self.load_figure_detail(&selected_figure_id);
        } else {
            self.selected_figure_id = None;
            self.selected_figure_detail = None;
        }
    }

    fn load_figure_detail(&mut self, figure_id: &str) {
        let Some(vault_root) = self.connected_vault_root.as_ref() else {
            self.error_message = Some("Open a vault first.".to_string());
            return;
        };

        match show_figure(ShowFigureRequest {
            vault_root: vault_root.clone(),
            figure_id: figure_id.to_string(),
        }) {
            Ok(detail) => {
                self.selected_figure_id = Some(figure_id.to_string());
                self.selected_figure_detail = Some(detail);
                self.figure_metadata_draft = None;
                self.source_metadata_draft = None;
                self.tag_mutation_draft = None;
                self.link_mutation_draft = None;
                self.delete_figure_draft = None;
                self.error_message = None;
            }
            Err(error) => {
                self.error_message = Some(error.to_string());
            }
        }
    }

    fn begin_figure_metadata_editing(&mut self) {
        let Some(detail) = self.selected_figure_detail.as_ref() else {
            self.error_message = Some("Select a figure to edit metadata.".to_string());
            return;
        };

        self.figure_metadata_draft = Some(FigureMetadataDraft {
            figure_id: detail.figure_id.clone(),
            original_display_name: detail.display_name.clone(),
            original_caption: detail.caption.clone(),
            display_name_input: detail.display_name.clone(),
            caption_input: detail.caption.clone().unwrap_or_default(),
            clear_caption: false,
            lifecycle: FigureEditorLifecycle::EditingClean,
            last_error: None,
        });
    }

    fn cancel_figure_metadata_editing(&mut self) {
        self.figure_metadata_draft = None;
        self.status_message = Some("Canceled figure metadata edits.".to_string());
    }

    fn begin_source_metadata_editing(&mut self) {
        let Some(detail) = self.selected_figure_detail.as_ref() else {
            self.error_message = Some("Select a figure to edit source metadata.".to_string());
            return;
        };

        let Some(source) = detail.sources.first() else {
            self.error_message = Some("Selected figure has no source metadata.".to_string());
            return;
        };

        self.source_metadata_draft = Some(SourceMetadataDraft {
            figure_id: detail.figure_id.clone(),
            original_title: source.source_title.clone(),
            original_authors: source.source_authors.clone(),
            original_published_at: source.source_published_at.clone(),
            title_input: source.source_title.clone().unwrap_or_default(),
            authors_input: source.source_authors.clone().unwrap_or_default(),
            published_at_input: source.source_published_at.clone().unwrap_or_default(),
            clear_title: false,
            clear_authors: false,
            clear_published_at: false,
            lifecycle: SourceEditorLifecycle::EditingClean,
            last_error: None,
        });
    }

    fn cancel_source_metadata_editing(&mut self) {
        self.source_metadata_draft = None;
        self.status_message = Some("Canceled source metadata edits.".to_string());
    }

    fn begin_tag_mutation_editing(&mut self) {
        let Some(detail) = self.selected_figure_detail.as_ref() else {
            self.error_message = Some("Select a figure to edit tags.".to_string());
            return;
        };

        self.tag_mutation_draft = Some(TagMutationDraft {
            figure_id: detail.figure_id.clone(),
            tag_input: String::new(),
            lifecycle: TagEditorLifecycle::EditingClean,
            last_error: None,
        });
    }

    fn cancel_tag_mutation_editing(&mut self) {
        self.tag_mutation_draft = None;
        self.status_message = Some("Canceled tag edits.".to_string());
    }

    fn begin_link_mutation_editing(&mut self) {
        let Some(detail) = self.selected_figure_detail.as_ref() else {
            self.error_message = Some("Select a figure to edit links.".to_string());
            return;
        };

        self.link_mutation_draft = Some(LinkMutationDraft {
            figure_id: detail.figure_id.clone(),
            to_figure_id_input: String::new(),
            relation_input: "related".to_string(),
            lifecycle: LinkEditorLifecycle::EditingClean,
            last_error: None,
        });
    }

    fn cancel_link_mutation_editing(&mut self) {
        self.link_mutation_draft = None;
        self.status_message = Some("Canceled link edits.".to_string());
    }

    fn begin_delete_figure_confirmation(&mut self) {
        let Some(detail) = self.selected_figure_detail.as_ref() else {
            self.error_message = Some("Select a figure to delete.".to_string());
            return;
        };

        self.delete_figure_draft = Some(DeleteFigureDraft {
            figure_id: detail.figure_id.clone(),
            lifecycle: DeleteEditorLifecycle::ConfirmingDelete,
            last_error: None,
        });
    }

    fn cancel_delete_figure_confirmation(&mut self) {
        self.delete_figure_draft = None;
        self.status_message = Some("Canceled delete action.".to_string());
    }

    fn capture_dropped_files(&mut self, context: &egui::Context) {
        let dropped_paths = context.input(|input| {
            input
                .raw
                .dropped_files
                .iter()
                .filter_map(|file| file.path.clone())
                .collect::<Vec<_>>()
        });

        if dropped_paths.is_empty() {
            return;
        }

        self.begin_drop_ingest_session(dropped_paths);
    }

    fn begin_drop_ingest_session(&mut self, input_paths: Vec<PathBuf>) {
        if input_paths.is_empty() {
            return;
        }

        let mut dropped_items = input_paths
            .into_iter()
            .map(|input_path| DropIngestItemDraft {
                normalized_path: normalize_drop_path(&input_path),
                input_path,
                source_type_input: String::new(),
                source_key_input: String::new(),
            })
            .collect::<Vec<_>>();
        dropped_items.sort_by(|left, right| left.normalized_path.cmp(&right.normalized_path));

        self.drop_ingest_session = DropIngestSessionDraft {
            lifecycle: DropIngestLifecycle::DropReceived,
            dropped_items,
            last_commit_results: Vec::new(),
            last_error: None,
        };
        self.error_message = None;
        self.sync_drop_ingest_lifecycle();
        self.status_message = Some(format!(
            "Drop session received {} file(s).",
            self.drop_ingest_session.dropped_items.len()
        ));
    }

    fn sync_drop_ingest_lifecycle(&mut self) {
        if self.drop_ingest_session.lifecycle == DropIngestLifecycle::Committing {
            return;
        }
        if self.drop_ingest_session.lifecycle == DropIngestLifecycle::Committed {
            return;
        }

        if self.drop_ingest_session.dropped_items.is_empty() {
            self.drop_ingest_session.lifecycle = DropIngestLifecycle::Idle;
            return;
        }

        let metadata_complete = self
            .drop_ingest_session
            .dropped_items
            .iter()
            .all(drop_ingest_metadata_is_complete);

        self.drop_ingest_session.lifecycle = if metadata_complete {
            DropIngestLifecycle::ReadyToCommit
        } else {
            DropIngestLifecycle::MetadataRequired
        };
    }

    fn begin_drop_ingest_commit(&mut self) {
        let Some(vault_root) = self.connected_vault_root.as_ref() else {
            self.error_message = Some("Open a vault first.".to_string());
            return;
        };
        if self.drop_ingest_session.lifecycle != DropIngestLifecycle::ReadyToCommit {
            self.error_message = Some("Drop session is not ready to commit.".to_string());
            return;
        }

        self.drop_ingest_session.lifecycle = DropIngestLifecycle::Committing;
        self.drop_ingest_session.last_commit_results.clear();
        self.drop_ingest_session.last_error = None;

        let mut commit_results = Vec::new();
        let mut imported_count = 0_usize;
        let mut skipped_count = 0_usize;
        let mut failed_count = 0_usize;

        for item in self.drop_ingest_session.dropped_items.clone() {
            let source_type = match parse_drop_source_type_input(&item.source_type_input) {
                Ok(value) => value,
                Err(error) => {
                    failed_count += 1;
                    commit_results.push(DropIngestItemCommitResult {
                        normalized_path: item.normalized_path,
                        status: DropIngestItemCommitStatus::Failed,
                        figure_id: None,
                        error: Some(error),
                    });
                    continue;
                }
            };

            match inject_figure(InjectRequest {
                vault_root: vault_root.clone(),
                file_path: item.input_path.clone(),
                source_type,
                source_key: item.source_key_input.trim().to_string(),
                copy_mode: CopyMode::Copy,
            }) {
                Ok(result) => {
                    if result.created_new {
                        imported_count += 1;
                        commit_results.push(DropIngestItemCommitResult {
                            normalized_path: item.normalized_path,
                            status: DropIngestItemCommitStatus::Imported,
                            figure_id: Some(result.figure_id),
                            error: None,
                        });
                    } else {
                        skipped_count += 1;
                        commit_results.push(DropIngestItemCommitResult {
                            normalized_path: item.normalized_path,
                            status: DropIngestItemCommitStatus::SkippedDuplicate,
                            figure_id: Some(result.figure_id),
                            error: None,
                        });
                    }
                }
                Err(error) => {
                    failed_count += 1;
                    commit_results.push(DropIngestItemCommitResult {
                        normalized_path: item.normalized_path,
                        status: DropIngestItemCommitStatus::Failed,
                        figure_id: None,
                        error: Some(error.to_string()),
                    });
                }
            }
        }

        self.drop_ingest_session.last_commit_results = commit_results;
        self.refresh_figure_rows();

        if failed_count > 0 {
            self.drop_ingest_session.lifecycle = DropIngestLifecycle::CommitFailed;
            let summary = format!(
                "Drop ingest completed with failures (imported={}, skipped={}, failed={}).",
                imported_count, skipped_count, failed_count
            );
            self.drop_ingest_session.last_error = Some(summary.clone());
            self.error_message = Some(summary);
            self.status_message = None;
        } else {
            self.drop_ingest_session.lifecycle = DropIngestLifecycle::Committed;
            self.drop_ingest_session.last_error = None;
            self.error_message = None;
            self.status_message = Some(format!(
                "Drop ingest committed (imported={}, skipped={}, failed=0).",
                imported_count, skipped_count
            ));
        }
    }

    fn clear_drop_ingest_session(&mut self) {
        self.drop_ingest_session = DropIngestSessionDraft::default();
        self.status_message = Some("Cleared drop session.".to_string());
    }

    fn save_figure_metadata_changes(&mut self, payload: FigureUpdatePayload) {
        if !payload.has_changes {
            self.status_message = Some("No figure metadata changes to save.".to_string());
            return;
        }

        let Some(vault_root) = self.connected_vault_root.as_ref() else {
            self.error_message = Some("Open a vault first.".to_string());
            return;
        };

        if let Some(draft) = self.figure_metadata_draft.as_mut() {
            draft.lifecycle = FigureEditorLifecycle::Saving;
            draft.last_error = None;
        }

        match update_figure(UpdateRequest {
            vault_root: vault_root.clone(),
            figure_id: payload.figure_id.clone(),
            name: payload.name,
            caption: payload.caption,
            clear_caption: payload.clear_caption,
            note_file: None,
        }) {
            Ok(result) => {
                self.error_message = None;
                self.status_message = Some(format!(
                    "Updated figure metadata: {} ({})",
                    result.figure_id,
                    result.updated_fields.join(", ")
                ));
                self.figure_metadata_draft = None;

                if payload.display_name_changed {
                    self.refresh_figure_rows();
                } else {
                    self.load_figure_detail(&payload.figure_id);
                }
            }
            Err(error) => {
                if let Some(draft) = self.figure_metadata_draft.as_mut() {
                    draft.lifecycle = FigureEditorLifecycle::SaveFailed;
                    draft.last_error = Some(error.to_string());
                }
                self.error_message = Some(error.to_string());
            }
        }
    }

    fn save_tag_mutation_changes(&mut self, payload: TagMutationPayload) {
        if !payload.has_changes {
            self.status_message = Some("No tag changes to save.".to_string());
            return;
        }

        let Some(vault_root) = self.connected_vault_root.as_ref() else {
            self.error_message = Some("Open a vault first.".to_string());
            return;
        };

        if let Some(draft) = self.tag_mutation_draft.as_mut() {
            draft.lifecycle = TagEditorLifecycle::Saving;
            draft.last_error = None;
        }

        let result = match payload.action {
            TagMutationAction::Add => add_tag_to_figure(AddTagRequest {
                vault_root: vault_root.clone(),
                figure_id: payload.figure_id.clone(),
                tag: payload.tag.clone(),
            })
            .map(|response| {
                if response.created_relation {
                    format!("Added tag: {}", response.normalized_tag)
                } else {
                    format!("Tag already assigned: {}", response.normalized_tag)
                }
            }),
            TagMutationAction::Remove => remove_tag_from_figure(RemoveTagRequest {
                vault_root: vault_root.clone(),
                figure_id: payload.figure_id.clone(),
                tag: payload.tag.clone(),
            })
            .map(|response| {
                if response.removed_relation {
                    format!("Removed tag: {}", response.normalized_tag)
                } else {
                    format!("Tag not assigned: {}", response.normalized_tag)
                }
            }),
        };

        match result {
            Ok(status_message) => {
                self.error_message = None;
                self.status_message = Some(status_message);
                self.tag_mutation_draft = None;
                self.load_figure_detail(&payload.figure_id);
            }
            Err(error) => {
                if let Some(draft) = self.tag_mutation_draft.as_mut() {
                    draft.lifecycle = TagEditorLifecycle::SaveFailed;
                    draft.last_error = Some(error.to_string());
                }
                self.error_message = Some(error.to_string());
            }
        }
    }

    fn save_link_mutation_changes(&mut self, payload: LinkMutationPayload) {
        if !payload.has_changes {
            self.status_message = Some("No link changes to save.".to_string());
            return;
        }

        let Some(vault_root) = self.connected_vault_root.as_ref() else {
            self.error_message = Some("Open a vault first.".to_string());
            return;
        };

        if let Some(draft) = self.link_mutation_draft.as_mut() {
            draft.lifecycle = LinkEditorLifecycle::Saving;
            draft.last_error = None;
        }

        let result = match payload.action {
            LinkMutationAction::Add => add_link(AddLinkRequest {
                vault_root: vault_root.clone(),
                from_figure_id: payload.figure_id.clone(),
                to_figure_id: payload.to_figure_id.clone(),
                relation: payload.relation.clone(),
            })
            .map(|response| {
                if response.created_link {
                    format!(
                        "Added link: {} -> {} [{}]",
                        response.from_figure_id,
                        response.to_figure_id,
                        response.normalized_relation
                    )
                } else {
                    format!(
                        "Link already exists: {} -> {} [{}]",
                        response.from_figure_id,
                        response.to_figure_id,
                        response.normalized_relation
                    )
                }
            }),
            LinkMutationAction::Remove => remove_link(RemoveLinkRequest {
                vault_root: vault_root.clone(),
                from_figure_id: payload.figure_id.clone(),
                to_figure_id: payload.to_figure_id.clone(),
            })
            .map(|response| {
                format!(
                    "Removed links: {} -> {} (count: {})",
                    response.from_figure_id, response.to_figure_id, response.removed_count
                )
            }),
        };

        match result {
            Ok(status_message) => {
                self.error_message = None;
                self.status_message = Some(status_message);
                self.link_mutation_draft = None;
                self.load_figure_detail(&payload.figure_id);
            }
            Err(error) => {
                if let Some(draft) = self.link_mutation_draft.as_mut() {
                    draft.lifecycle = LinkEditorLifecycle::SaveFailed;
                    draft.last_error = Some(error.to_string());
                }
                self.error_message = Some(error.to_string());
            }
        }
    }

    fn confirm_delete_figure(&mut self, payload: DeleteFigurePayload) {
        let Some(vault_root) = self.connected_vault_root.as_ref() else {
            self.error_message = Some("Open a vault first.".to_string());
            return;
        };

        if let Some(draft) = self.delete_figure_draft.as_mut() {
            draft.lifecycle = DeleteEditorLifecycle::Deleting;
            draft.last_error = None;
        }

        let deleted_row_index = self
            .figure_rows
            .iter()
            .position(|row| row.figure_id == payload.figure_id);

        match delete_figure(DeleteFigureRequest {
            vault_root: vault_root.clone(),
            figure_id: payload.figure_id.clone(),
        }) {
            Ok(result) => {
                self.error_message = None;
                self.figure_metadata_draft = None;
                self.source_metadata_draft = None;
                self.tag_mutation_draft = None;
                self.link_mutation_draft = None;
                self.delete_figure_draft = None;
                self.selected_figure_id = None;
                self.selected_figure_detail = None;
                self.refresh_figure_rows();
                self.apply_post_delete_selection(deleted_row_index);
                self.status_message = Some(format!(
                    "Deleted figure: {} (removed_orphan_tags={}, removed_managed_file={})",
                    result.figure_id, result.removed_orphan_tag_count, result.removed_managed_file
                ));
            }
            Err(error) => {
                if let Some(draft) = self.delete_figure_draft.as_mut() {
                    draft.lifecycle = DeleteEditorLifecycle::DeleteFailed;
                    draft.last_error = Some(error.to_string());
                }
                self.error_message = Some(error.to_string());
            }
        }
    }

    fn apply_post_delete_selection(&mut self, deleted_row_index: Option<usize>) {
        if self.figure_rows.is_empty() {
            self.selected_figure_id = None;
            self.selected_figure_detail = None;
            return;
        }

        let next_index = match deleted_row_index {
            Some(index) if index < self.figure_rows.len() => index,
            Some(_) => self.figure_rows.len().saturating_sub(1),
            None => 0,
        };
        let next_figure_id = self.figure_rows[next_index].figure_id.clone();
        self.load_figure_detail(&next_figure_id);
    }

    fn save_source_metadata_changes(&mut self, payload: SourceUpdatePayload) {
        if !payload.has_changes {
            self.status_message = Some("No source metadata changes to save.".to_string());
            return;
        }

        let Some(vault_root) = self.connected_vault_root.as_ref() else {
            self.error_message = Some("Open a vault first.".to_string());
            return;
        };

        if let Some(draft) = self.source_metadata_draft.as_mut() {
            draft.lifecycle = SourceEditorLifecycle::Saving;
            draft.last_error = None;
        }

        match update_source_metadata(UpdateSourceRequest {
            vault_root: vault_root.clone(),
            figure_id: payload.figure_id.clone(),
            title: payload.title,
            authors: payload.authors,
            published_at: payload.published_at,
            clear_title: payload.clear_title,
            clear_authors: payload.clear_authors,
            clear_published_at: payload.clear_published_at,
        }) {
            Ok(result) => {
                self.error_message = None;
                self.status_message = Some(format!(
                    "Updated source metadata: {} ({})",
                    result.figure_id,
                    result.updated_fields.join(", ")
                ));
                self.source_metadata_draft = None;
                self.load_figure_detail(&payload.figure_id);
            }
            Err(error) => {
                if let Some(draft) = self.source_metadata_draft.as_mut() {
                    draft.lifecycle = SourceEditorLifecycle::SaveFailed;
                    draft.last_error = Some(error.to_string());
                }
                self.error_message = Some(error.to_string());
            }
        }
    }

    fn render_top_controls(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Vault:");
            ui.text_edit_singleline(&mut self.vault_root_input);
            if ui.button("Open Vault").clicked() {
                self.connect_and_refresh();
            }
            if ui.button("Refresh").clicked() {
                self.refresh_figure_rows();
            }
        });

        ui.horizontal(|ui| {
            ui.label("Search text:");
            ui.text_edit_singleline(&mut self.search_text);
            if ui.button("Run Search").clicked() {
                self.refresh_figure_rows();
            }
            if ui.button("Clear Search").clicked() {
                self.search_text.clear();
                self.refresh_figure_rows();
            }
        });

        ui.separator();
        self.render_drop_ingest_panel(ui);

        if let Some(vault_root) = self.connected_vault_root.as_ref() {
            ui.label(format!("Connected vault: {}", vault_root.display()));
        }

        if let Some(status_message) = self.status_message.as_ref() {
            ui.label(status_message);
        }
        if let Some(error_message) = self.error_message.as_ref() {
            ui.colored_label(egui::Color32::RED, error_message);
        }
    }

    fn render_drop_ingest_panel(&mut self, ui: &mut egui::Ui) {
        ui.label("Drag-and-Drop Ingest Session");
        ui.label(format!(
            "Session state: {}",
            drop_ingest_lifecycle_label(self.drop_ingest_session.lifecycle)
        ));

        if let Some(error_message) = self.drop_ingest_session.last_error.as_ref() {
            ui.colored_label(egui::Color32::RED, error_message);
        }

        if self.drop_ingest_session.dropped_items.is_empty() {
            ui.label("Drop file(s) onto the window to start a session.");
            return;
        }

        for item in &mut self.drop_ingest_session.dropped_items {
            ui.separator();
            ui.label(format!("Path: {}", item.input_path.display()));
            ui.horizontal(|ui| {
                ui.label("Source type:");
                ui.text_edit_singleline(&mut item.source_type_input);
            });
            ui.horizontal(|ui| {
                ui.label("Source key:");
                ui.text_edit_singleline(&mut item.source_key_input);
            });
        }

        self.sync_drop_ingest_lifecycle();
        let can_commit = self.drop_ingest_session.lifecycle == DropIngestLifecycle::ReadyToCommit;
        let is_committing = self.drop_ingest_session.lifecycle == DropIngestLifecycle::Committing;
        ui.horizontal(|ui| {
            if ui
                .add_enabled(!is_committing, egui::Button::new("Commit Drop Session"))
                .clicked()
            {
                self.begin_drop_ingest_commit();
            }
            if ui
                .add_enabled(!is_committing, egui::Button::new("Clear Drop Session"))
                .clicked()
            {
                self.clear_drop_ingest_session();
            }
        });

        if !can_commit {
            ui.colored_label(
                egui::Color32::YELLOW,
                "All dropped items require non-empty source type and source key before commit.",
            );
        }

        if !self.drop_ingest_session.last_commit_results.is_empty() {
            ui.separator();
            ui.label("Last commit results:");
            for item_result in &self.drop_ingest_session.last_commit_results {
                let status_label = match item_result.status {
                    DropIngestItemCommitStatus::Imported => "imported",
                    DropIngestItemCommitStatus::SkippedDuplicate => "skipped_duplicate",
                    DropIngestItemCommitStatus::Failed => "failed",
                };
                let mut line = format!("{} | {}", item_result.normalized_path, status_label);
                if let Some(figure_id) = item_result.figure_id.as_deref() {
                    line.push_str(&format!(" | figure_id={figure_id}"));
                }
                if let Some(error_message) = item_result.error.as_deref() {
                    line.push_str(&format!(" | error={error_message}"));
                }
                ui.label(line);
            }
        }
    }

    fn render_figure_list_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Figures");
        let mut pending_selection = None;

        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("figure_rows_grid")
                .num_columns(4)
                .striped(true)
                .show(ui, |ui| {
                    ui.strong("Figure ID");
                    ui.strong("Display Name");
                    ui.strong("Created At");
                    ui.strong("Updated At");
                    ui.end_row();

                    for row in &self.figure_rows {
                        let is_selected =
                            self.selected_figure_id.as_deref() == Some(row.figure_id.as_str());
                        if ui.selectable_label(is_selected, &row.figure_id).clicked() {
                            pending_selection = Some(row.figure_id.clone());
                        }
                        ui.label(&row.display_name);
                        ui.label(row.created_at.as_deref().unwrap_or("-"));
                        ui.label(row.updated_at.as_deref().unwrap_or("-"));
                        ui.end_row();
                    }
                });
        });

        if let Some(figure_id) = pending_selection {
            self.load_figure_detail(&figure_id);
        }
    }

    fn render_figure_detail_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Figure Detail");

        let Some(detail) = self.selected_figure_detail.clone() else {
            ui.label("Select a figure to view detail.");
            return;
        };

        if self
            .figure_metadata_draft
            .as_ref()
            .is_some_and(|draft| draft.figure_id != detail.figure_id)
        {
            self.figure_metadata_draft = None;
        }
        if self
            .source_metadata_draft
            .as_ref()
            .is_some_and(|draft| draft.figure_id != detail.figure_id)
        {
            self.source_metadata_draft = None;
        }
        if self
            .tag_mutation_draft
            .as_ref()
            .is_some_and(|draft| draft.figure_id != detail.figure_id)
        {
            self.tag_mutation_draft = None;
        }
        if self
            .link_mutation_draft
            .as_ref()
            .is_some_and(|draft| draft.figure_id != detail.figure_id)
        {
            self.link_mutation_draft = None;
        }
        if self
            .delete_figure_draft
            .as_ref()
            .is_some_and(|draft| draft.figure_id != detail.figure_id)
        {
            self.delete_figure_draft = None;
        }

        let mut pending_action: Option<EditorAction> = None;

        ui.label(format!("Figure ID: {}", detail.figure_id));
        ui.label("Delete Figure");
        if let Some(draft) = self.delete_figure_draft.as_mut() {
            ui.label(format!(
                "Delete state: {}",
                delete_lifecycle_label(draft.lifecycle)
            ));
            ui.colored_label(
                egui::Color32::YELLOW,
                "Confirm delete to remove this figure and related references.",
            );

            if let Some(error_message) = draft.last_error.as_ref() {
                ui.colored_label(egui::Color32::RED, error_message);
            }

            let is_deleting = draft.lifecycle == DeleteEditorLifecycle::Deleting;
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(!is_deleting, egui::Button::new("Confirm Delete"))
                    .clicked()
                {
                    pending_action = Some(EditorAction::DeleteConfirm(DeleteFigurePayload {
                        figure_id: draft.figure_id.clone(),
                    }));
                }
                if ui
                    .add_enabled(!is_deleting, egui::Button::new("Cancel Delete"))
                    .clicked()
                {
                    pending_action = Some(EditorAction::DeleteCancel);
                }
            });
        } else if ui.button("Delete Figure").clicked() {
            self.begin_delete_figure_confirmation();
        }

        if let Some(draft) = self.figure_metadata_draft.as_mut() {
            let payload = build_figure_update_payload(draft);
            sync_figure_draft_lifecycle(draft, payload.has_changes);

            ui.separator();
            ui.label("Figure Metadata Editor");
            ui.label(format!(
                "Editor state: {}",
                lifecycle_label(draft.lifecycle)
            ));

            let is_saving = draft.lifecycle == FigureEditorLifecycle::Saving;
            ui.add_enabled_ui(!is_saving, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Display Name:");
                    ui.text_edit_singleline(&mut draft.display_name_input);
                });
                ui.horizontal(|ui| {
                    ui.label("Caption:");
                    ui.text_edit_singleline(&mut draft.caption_input);
                });
                ui.checkbox(&mut draft.clear_caption, "Clear caption");
            });

            if let Some(error_message) = draft.last_error.as_ref() {
                ui.colored_label(egui::Color32::RED, error_message);
            }

            let can_save = payload.has_changes && !is_saving;
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(can_save, egui::Button::new("Save"))
                    .clicked()
                {
                    pending_action = Some(EditorAction::FigureSave(payload.clone()));
                }
                if ui
                    .add_enabled(!is_saving, egui::Button::new("Cancel"))
                    .clicked()
                {
                    pending_action = Some(EditorAction::FigureCancel);
                }
            });
        } else {
            ui.label(format!("Display Name: {}", detail.display_name));
            ui.label(format!(
                "Caption: {}",
                detail.caption.as_deref().unwrap_or("-")
            ));
            if ui.button("Edit Figure Metadata").clicked() {
                self.begin_figure_metadata_editing();
            }
        }

        ui.separator();
        ui.label(format!("File Path: {}", detail.file_path));
        ui.label(format!("Media Type: {}", detail.media_type));
        ui.label(format!("File Size (bytes): {}", detail.file_size_bytes));
        ui.label(format!("SHA256: {}", detail.file_hash_sha256));
        ui.label(format!("Created At: {}", detail.created_at));
        ui.label(format!("Updated At: {}", detail.updated_at));

        ui.separator();
        ui.label(format!("Tags: {}", detail.tags.join(", ")));
        ui.label("Tag Editor");
        if let Some(draft) = self.tag_mutation_draft.as_mut() {
            let add_payload = build_tag_mutation_payload(draft, TagMutationAction::Add);
            let remove_payload = build_tag_mutation_payload(draft, TagMutationAction::Remove);
            sync_tag_draft_lifecycle(draft, add_payload.has_changes);
            ui.label(format!(
                "Editor state: {}",
                tag_lifecycle_label(draft.lifecycle)
            ));

            let is_saving = draft.lifecycle == TagEditorLifecycle::Saving;
            ui.add_enabled_ui(!is_saving, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Tag:");
                    ui.text_edit_singleline(&mut draft.tag_input);
                });
            });

            if let Some(error_message) = draft.last_error.as_ref() {
                ui.colored_label(egui::Color32::RED, error_message);
            }

            let can_save = add_payload.has_changes && !is_saving;
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(can_save, egui::Button::new("Add Tag"))
                    .clicked()
                {
                    pending_action = Some(EditorAction::TagSave(add_payload.clone()));
                }
                if ui
                    .add_enabled(can_save, egui::Button::new("Remove Tag"))
                    .clicked()
                {
                    pending_action = Some(EditorAction::TagSave(remove_payload.clone()));
                }
                if ui
                    .add_enabled(!is_saving, egui::Button::new("Cancel"))
                    .clicked()
                {
                    pending_action = Some(EditorAction::TagCancel);
                }
            });
        } else if ui.button("Edit Tags").clicked() {
            self.begin_tag_mutation_editing();
        }

        ui.separator();
        ui.label("Sources:");
        for source in &detail.sources {
            ui.label(format!(
                "- {} | {} | title={} | authors={} | published_at={}",
                source.source_type,
                source.source_key,
                source.source_title.as_deref().unwrap_or("-"),
                source.source_authors.as_deref().unwrap_or("-"),
                source.source_published_at.as_deref().unwrap_or("-")
            ));
        }

        ui.separator();
        ui.label("Source Metadata Editor");
        if let Some(draft) = self.source_metadata_draft.as_mut() {
            let payload = build_source_update_payload(draft);
            sync_source_draft_lifecycle(draft, payload.has_changes);
            ui.label(format!(
                "Editor state: {}",
                source_lifecycle_label(draft.lifecycle)
            ));

            let is_saving = draft.lifecycle == SourceEditorLifecycle::Saving;
            ui.add_enabled_ui(!is_saving, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Title:");
                    ui.text_edit_singleline(&mut draft.title_input);
                });
                ui.checkbox(&mut draft.clear_title, "Clear title");
                ui.horizontal(|ui| {
                    ui.label("Authors:");
                    ui.text_edit_singleline(&mut draft.authors_input);
                });
                ui.checkbox(&mut draft.clear_authors, "Clear authors");
                ui.horizontal(|ui| {
                    ui.label("Published At:");
                    ui.text_edit_singleline(&mut draft.published_at_input);
                });
                ui.checkbox(&mut draft.clear_published_at, "Clear published at");
            });

            if let Some(error_message) = draft.last_error.as_ref() {
                ui.colored_label(egui::Color32::RED, error_message);
            }

            let can_save = payload.has_changes && !is_saving;
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(can_save, egui::Button::new("Save Source Metadata"))
                    .clicked()
                {
                    pending_action = Some(EditorAction::SourceSave(payload.clone()));
                }
                if ui
                    .add_enabled(!is_saving, egui::Button::new("Cancel"))
                    .clicked()
                {
                    pending_action = Some(EditorAction::SourceCancel);
                }
            });
        } else if ui.button("Edit Source Metadata").clicked() {
            self.begin_source_metadata_editing();
        }

        ui.separator();
        ui.label("Outbound links:");
        for link in &detail.outbound_links {
            ui.label(format!(
                "- {} [{}] at {}",
                link.to_figure_id, link.relation_type, link.created_at
            ));
        }
        ui.label("Link Editor");
        if let Some(draft) = self.link_mutation_draft.as_mut() {
            let add_payload = build_link_mutation_payload(draft, LinkMutationAction::Add);
            let remove_payload = build_link_mutation_payload(draft, LinkMutationAction::Remove);
            sync_link_draft_lifecycle(draft, add_payload.has_changes);
            ui.label(format!(
                "Editor state: {}",
                link_lifecycle_label(draft.lifecycle)
            ));

            let is_saving = draft.lifecycle == LinkEditorLifecycle::Saving;
            ui.add_enabled_ui(!is_saving, |ui| {
                ui.horizontal(|ui| {
                    ui.label("To Figure ID:");
                    ui.text_edit_singleline(&mut draft.to_figure_id_input);
                });
                ui.horizontal(|ui| {
                    ui.label("Relation:");
                    ui.text_edit_singleline(&mut draft.relation_input);
                });
            });

            if let Some(error_message) = draft.last_error.as_ref() {
                ui.colored_label(egui::Color32::RED, error_message);
            }

            let can_save = add_payload.has_changes && !is_saving;
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(can_save, egui::Button::new("Add Link"))
                    .clicked()
                {
                    pending_action = Some(EditorAction::LinkSave(add_payload.clone()));
                }
                if ui
                    .add_enabled(can_save, egui::Button::new("Remove Link"))
                    .clicked()
                {
                    pending_action = Some(EditorAction::LinkSave(remove_payload.clone()));
                }
                if ui
                    .add_enabled(!is_saving, egui::Button::new("Cancel"))
                    .clicked()
                {
                    pending_action = Some(EditorAction::LinkCancel);
                }
            });
        } else if ui.button("Edit Links").clicked() {
            self.begin_link_mutation_editing();
        }

        if let Some(action) = pending_action {
            match action {
                EditorAction::FigureSave(payload) => self.save_figure_metadata_changes(payload),
                EditorAction::FigureCancel => self.cancel_figure_metadata_editing(),
                EditorAction::SourceSave(payload) => self.save_source_metadata_changes(payload),
                EditorAction::SourceCancel => self.cancel_source_metadata_editing(),
                EditorAction::TagSave(payload) => self.save_tag_mutation_changes(payload),
                EditorAction::TagCancel => self.cancel_tag_mutation_editing(),
                EditorAction::LinkSave(payload) => self.save_link_mutation_changes(payload),
                EditorAction::LinkCancel => self.cancel_link_mutation_editing(),
                EditorAction::DeleteConfirm(payload) => self.confirm_delete_figure(payload),
                EditorAction::DeleteCancel => self.cancel_delete_figure_confirmation(),
            }
        }

        ui.separator();
        ui.label("Note:");
        let mut note_text = detail
            .note
            .as_ref()
            .map(|note| note.note_markdown.clone())
            .unwrap_or_else(|| "-".to_string());
        ui.add(
            egui::TextEdit::multiline(&mut note_text)
                .desired_rows(8)
                .interactive(false),
        );
    }
}

#[derive(Debug, Clone)]
enum EditorAction {
    FigureSave(FigureUpdatePayload),
    FigureCancel,
    SourceSave(SourceUpdatePayload),
    SourceCancel,
    TagSave(TagMutationPayload),
    TagCancel,
    LinkSave(LinkMutationPayload),
    LinkCancel,
    DeleteConfirm(DeleteFigurePayload),
    DeleteCancel,
}

impl eframe::App for LamianGuiApp {
    fn update(&mut self, context: &egui::Context, _frame: &mut eframe::Frame) {
        self.capture_dropped_files(context);

        egui::TopBottomPanel::top("top_controls").show(context, |ui| {
            self.render_top_controls(ui);
        });

        egui::CentralPanel::default().show(context, |ui| {
            ui.columns(2, |columns| {
                self.render_figure_list_panel(&mut columns[0]);
                self.render_figure_detail_panel(&mut columns[1]);
            });
        });
    }
}

fn figure_rows_from_list(rows: Vec<ListFigureRow>) -> Vec<FigureListRowView> {
    rows.into_iter()
        .map(|row| FigureListRowView {
            figure_id: row.figure_id,
            display_name: row.display_name,
            created_at: Some(row.created_at),
            updated_at: Some(row.updated_at),
        })
        .collect()
}

fn figure_rows_from_search(rows: Vec<SearchFigure>) -> Vec<FigureListRowView> {
    rows.into_iter()
        .map(|row| FigureListRowView {
            figure_id: row.figure_id,
            display_name: row.display_name,
            created_at: None,
            updated_at: None,
        })
        .collect()
}

fn build_figure_update_payload(draft: &FigureMetadataDraft) -> FigureUpdatePayload {
    let original_display_name = draft.original_display_name.trim().to_string();
    let display_name_input_trimmed = draft.display_name_input.trim().to_string();
    let display_name_changed = if display_name_input_trimmed.is_empty() {
        draft.display_name_input != draft.original_display_name
    } else {
        display_name_input_trimmed != original_display_name
    };

    let name = if display_name_changed {
        Some(draft.display_name_input.clone())
    } else {
        None
    };

    let original_caption = draft
        .original_caption
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let caption_input_trimmed = draft.caption_input.trim();
    let caption_changed_without_clear = !draft.clear_caption
        && if caption_input_trimmed.is_empty() {
            draft.original_caption.is_some()
                && draft.caption_input != draft.original_caption.clone().unwrap_or_default()
        } else {
            Some(caption_input_trimmed) != original_caption
        };

    let caption = if caption_changed_without_clear {
        Some(draft.caption_input.clone())
    } else {
        None
    };

    let clear_caption = draft.clear_caption;
    let clear_caption_changed = clear_caption && draft.original_caption.is_some();
    let has_changes =
        display_name_changed || caption_changed_without_clear || clear_caption_changed;

    FigureUpdatePayload {
        figure_id: draft.figure_id.clone(),
        name,
        caption,
        clear_caption,
        has_changes,
        display_name_changed,
    }
}

fn sync_figure_draft_lifecycle(draft: &mut FigureMetadataDraft, has_changes: bool) {
    if draft.lifecycle == FigureEditorLifecycle::Saving {
        return;
    }

    draft.lifecycle = if has_changes {
        FigureEditorLifecycle::EditingDirty
    } else {
        FigureEditorLifecycle::EditingClean
    };
}

fn lifecycle_label(lifecycle: FigureEditorLifecycle) -> &'static str {
    match lifecycle {
        FigureEditorLifecycle::EditingClean => "editing_clean",
        FigureEditorLifecycle::EditingDirty => "editing_dirty",
        FigureEditorLifecycle::Saving => "saving",
        FigureEditorLifecycle::SaveFailed => "save_failed",
    }
}

fn build_source_update_payload(draft: &SourceMetadataDraft) -> SourceUpdatePayload {
    let title_changed = source_field_changed(
        draft.original_title.as_deref(),
        &draft.title_input,
        draft.clear_title,
    );
    let authors_changed = source_field_changed(
        draft.original_authors.as_deref(),
        &draft.authors_input,
        draft.clear_authors,
    );
    let published_at_changed = source_field_changed(
        draft.original_published_at.as_deref(),
        &draft.published_at_input,
        draft.clear_published_at,
    );

    SourceUpdatePayload {
        figure_id: draft.figure_id.clone(),
        title: if title_changed && !draft.clear_title {
            Some(draft.title_input.clone())
        } else {
            None
        },
        authors: if authors_changed && !draft.clear_authors {
            Some(draft.authors_input.clone())
        } else {
            None
        },
        published_at: if published_at_changed && !draft.clear_published_at {
            Some(draft.published_at_input.clone())
        } else {
            None
        },
        clear_title: draft.clear_title,
        clear_authors: draft.clear_authors,
        clear_published_at: draft.clear_published_at,
        has_changes: title_changed || authors_changed || published_at_changed,
    }
}

fn source_field_changed(
    original_value: Option<&str>,
    input_value: &str,
    clear_value: bool,
) -> bool {
    if clear_value {
        return original_value.is_some();
    }

    let original_trimmed = original_value
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let input_trimmed = input_value.trim();
    if input_trimmed.is_empty() {
        original_trimmed.is_some()
    } else {
        Some(input_trimmed) != original_trimmed
    }
}

fn sync_source_draft_lifecycle(draft: &mut SourceMetadataDraft, has_changes: bool) {
    if draft.lifecycle == SourceEditorLifecycle::Saving {
        return;
    }

    draft.lifecycle = if has_changes {
        SourceEditorLifecycle::EditingDirty
    } else {
        SourceEditorLifecycle::EditingClean
    };
}

fn source_lifecycle_label(lifecycle: SourceEditorLifecycle) -> &'static str {
    match lifecycle {
        SourceEditorLifecycle::EditingClean => "editing_clean",
        SourceEditorLifecycle::EditingDirty => "editing_dirty",
        SourceEditorLifecycle::Saving => "saving",
        SourceEditorLifecycle::SaveFailed => "save_failed",
    }
}

fn build_tag_mutation_payload(
    draft: &TagMutationDraft,
    action: TagMutationAction,
) -> TagMutationPayload {
    let tag = draft.tag_input.trim().to_string();
    TagMutationPayload {
        figure_id: draft.figure_id.clone(),
        tag,
        action,
        has_changes: !draft.tag_input.trim().is_empty(),
    }
}

fn sync_tag_draft_lifecycle(draft: &mut TagMutationDraft, has_changes: bool) {
    if draft.lifecycle == TagEditorLifecycle::Saving {
        return;
    }

    draft.lifecycle = if has_changes {
        TagEditorLifecycle::EditingDirty
    } else {
        TagEditorLifecycle::EditingClean
    };
}

fn tag_lifecycle_label(lifecycle: TagEditorLifecycle) -> &'static str {
    match lifecycle {
        TagEditorLifecycle::EditingClean => "editing_clean",
        TagEditorLifecycle::EditingDirty => "editing_dirty",
        TagEditorLifecycle::Saving => "saving",
        TagEditorLifecycle::SaveFailed => "save_failed",
    }
}

fn build_link_mutation_payload(
    draft: &LinkMutationDraft,
    action: LinkMutationAction,
) -> LinkMutationPayload {
    let to_figure_id = draft.to_figure_id_input.trim().to_string();
    LinkMutationPayload {
        figure_id: draft.figure_id.clone(),
        to_figure_id: to_figure_id.clone(),
        relation: draft.relation_input.trim().to_string(),
        action,
        has_changes: !to_figure_id.is_empty(),
    }
}

fn sync_link_draft_lifecycle(draft: &mut LinkMutationDraft, has_changes: bool) {
    if draft.lifecycle == LinkEditorLifecycle::Saving {
        return;
    }

    draft.lifecycle = if has_changes {
        LinkEditorLifecycle::EditingDirty
    } else {
        LinkEditorLifecycle::EditingClean
    };
}

fn link_lifecycle_label(lifecycle: LinkEditorLifecycle) -> &'static str {
    match lifecycle {
        LinkEditorLifecycle::EditingClean => "editing_clean",
        LinkEditorLifecycle::EditingDirty => "editing_dirty",
        LinkEditorLifecycle::Saving => "saving",
        LinkEditorLifecycle::SaveFailed => "save_failed",
    }
}

fn delete_lifecycle_label(lifecycle: DeleteEditorLifecycle) -> &'static str {
    match lifecycle {
        DeleteEditorLifecycle::ConfirmingDelete => "confirming_delete",
        DeleteEditorLifecycle::Deleting => "deleting",
        DeleteEditorLifecycle::DeleteFailed => "delete_failed",
    }
}

fn normalize_drop_path(path: &Path) -> String {
    path.components()
        .filter_map(|component| {
            let segment = component.as_os_str().to_string_lossy().replace('\\', "/");
            if segment.is_empty() {
                None
            } else {
                Some(segment)
            }
        })
        .collect::<Vec<_>>()
        .join("/")
}

fn parse_drop_source_type_input(value: &str) -> Result<SourceType, String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "doi" => Ok(SourceType::Doi),
        "url" => Ok(SourceType::Url),
        "local" => Ok(SourceType::Local),
        "manual" => Ok(SourceType::Manual),
        _ => Err(format!(
            "invalid source type `{}`; expected doi|url|local|manual",
            value.trim()
        )),
    }
}

fn drop_ingest_metadata_is_complete(item: &DropIngestItemDraft) -> bool {
    !item.source_type_input.trim().is_empty() && !item.source_key_input.trim().is_empty()
}

fn drop_ingest_lifecycle_label(lifecycle: DropIngestLifecycle) -> &'static str {
    match lifecycle {
        DropIngestLifecycle::Idle => "idle",
        DropIngestLifecycle::DropReceived => "drop_received",
        DropIngestLifecycle::MetadataRequired => "metadata_required",
        DropIngestLifecycle::ReadyToCommit => "ready_to_commit",
        DropIngestLifecycle::Committing => "committing",
        DropIngestLifecycle::Committed => "committed",
        DropIngestLifecycle::CommitFailed => "commit_failed",
    }
}

#[cfg(test)]
mod tests {
    use super::{
        build_figure_update_payload, build_link_mutation_payload, build_source_update_payload,
        build_tag_mutation_payload, drop_ingest_lifecycle_label, figure_rows_from_list,
        figure_rows_from_search, normalize_drop_path, parse_drop_source_type_input,
        sync_figure_draft_lifecycle, sync_link_draft_lifecycle, sync_source_draft_lifecycle,
        sync_tag_draft_lifecycle, DeleteEditorLifecycle, DeleteFigurePayload, DropIngestLifecycle,
        FigureEditorLifecycle, FigureMetadataDraft, LamianGuiApp, LinkEditorLifecycle,
        LinkMutationAction, LinkMutationDraft, SourceEditorLifecycle, SourceMetadataDraft,
        TagEditorLifecycle, TagMutationAction, TagMutationDraft,
    };
    use std::fs;
    use std::path::{Path, PathBuf};

    use tempfile::TempDir;

    use crate::db;
    use crate::delete::{delete_figure, DeleteFigureRequest};
    use crate::inject::{inject_figure, CopyMode, InjectRequest, SourceType};
    use crate::list::ListFigureRow;
    use crate::search::SearchFigure;
    use crate::source::{update_source_metadata, UpdateSourceRequest};

    #[test]
    fn rows_from_list_keep_input_order() {
        let rows = vec![
            ListFigureRow {
                figure_id: "fig_b".to_string(),
                display_name: "B".to_string(),
                created_at: "2026-02-24T00:00:00Z".to_string(),
                updated_at: "2026-02-24T00:00:00Z".to_string(),
            },
            ListFigureRow {
                figure_id: "fig_a".to_string(),
                display_name: "A".to_string(),
                created_at: "2026-02-24T00:00:01Z".to_string(),
                updated_at: "2026-02-24T00:00:01Z".to_string(),
            },
        ];

        let result = figure_rows_from_list(rows);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].figure_id, "fig_b");
        assert_eq!(result[1].figure_id, "fig_a");
        assert_eq!(
            result[0].created_at.as_deref(),
            Some("2026-02-24T00:00:00Z")
        );
        assert_eq!(
            result[0].updated_at.as_deref(),
            Some("2026-02-24T00:00:00Z")
        );
    }

    #[test]
    fn rows_from_search_keep_input_order() {
        let rows = vec![
            SearchFigure {
                figure_id: "fig_1".to_string(),
                display_name: "one".to_string(),
            },
            SearchFigure {
                figure_id: "fig_2".to_string(),
                display_name: "two".to_string(),
            },
        ];

        let result = figure_rows_from_search(rows);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].figure_id, "fig_1");
        assert_eq!(result[1].figure_id, "fig_2");
        assert!(result[0].created_at.is_none());
        assert!(result[0].updated_at.is_none());
    }

    #[test]
    fn lamian_error_is_displayable() {
        let error = crate::error::LamianError::MissingSearchField { field: "text" };
        assert!(error.to_string().contains("text"));
    }

    #[test]
    fn build_figure_update_payload_reports_name_change() {
        let draft = FigureMetadataDraft {
            figure_id: "fig_1".to_string(),
            original_display_name: "old".to_string(),
            original_caption: Some("caption".to_string()),
            display_name_input: "new".to_string(),
            caption_input: "caption".to_string(),
            clear_caption: false,
            lifecycle: FigureEditorLifecycle::EditingDirty,
            last_error: None,
        };

        let payload = build_figure_update_payload(&draft);

        assert!(payload.has_changes);
        assert!(payload.display_name_changed);
        assert_eq!(payload.name.as_deref(), Some("new"));
        assert_eq!(payload.caption, None);
        assert!(!payload.clear_caption);
    }

    #[test]
    fn build_figure_update_payload_reports_clear_caption_change() {
        let draft = FigureMetadataDraft {
            figure_id: "fig_2".to_string(),
            original_display_name: "name".to_string(),
            original_caption: Some("caption".to_string()),
            display_name_input: "name".to_string(),
            caption_input: "caption".to_string(),
            clear_caption: true,
            lifecycle: FigureEditorLifecycle::EditingDirty,
            last_error: None,
        };

        let payload = build_figure_update_payload(&draft);

        assert!(payload.has_changes);
        assert_eq!(payload.name, None);
        assert_eq!(payload.caption, None);
        assert!(payload.clear_caption);
    }

    #[test]
    fn build_figure_update_payload_reports_no_change_when_draft_matches_detail() {
        let draft = FigureMetadataDraft {
            figure_id: "fig_3".to_string(),
            original_display_name: "name".to_string(),
            original_caption: None,
            display_name_input: "name".to_string(),
            caption_input: "".to_string(),
            clear_caption: false,
            lifecycle: FigureEditorLifecycle::EditingClean,
            last_error: None,
        };

        let payload = build_figure_update_payload(&draft);

        assert!(!payload.has_changes);
        assert_eq!(payload.name, None);
        assert_eq!(payload.caption, None);
        assert!(!payload.clear_caption);
    }

    #[test]
    fn build_source_update_payload_reports_no_change_for_matching_values() {
        let draft = SourceMetadataDraft {
            figure_id: "fig_4".to_string(),
            original_title: Some("Title".to_string()),
            original_authors: Some("Authors".to_string()),
            original_published_at: Some("2026-02-25".to_string()),
            title_input: "Title".to_string(),
            authors_input: "Authors".to_string(),
            published_at_input: "2026-02-25".to_string(),
            clear_title: false,
            clear_authors: false,
            clear_published_at: false,
            lifecycle: SourceEditorLifecycle::EditingClean,
            last_error: None,
        };

        let payload = build_source_update_payload(&draft);

        assert!(!payload.has_changes);
        assert_eq!(payload.title, None);
        assert_eq!(payload.authors, None);
        assert_eq!(payload.published_at, None);
        assert!(!payload.clear_title);
        assert!(!payload.clear_authors);
        assert!(!payload.clear_published_at);
    }

    #[test]
    fn build_source_update_payload_reports_changed_title() {
        let draft = SourceMetadataDraft {
            figure_id: "fig_5".to_string(),
            original_title: Some("Old".to_string()),
            original_authors: None,
            original_published_at: None,
            title_input: "New".to_string(),
            authors_input: "".to_string(),
            published_at_input: "".to_string(),
            clear_title: false,
            clear_authors: false,
            clear_published_at: false,
            lifecycle: SourceEditorLifecycle::EditingDirty,
            last_error: None,
        };

        let payload = build_source_update_payload(&draft);

        assert!(payload.has_changes);
        assert_eq!(payload.title.as_deref(), Some("New"));
        assert_eq!(payload.authors, None);
        assert_eq!(payload.published_at, None);
    }

    #[test]
    fn build_source_update_payload_reports_clear_flags() {
        let draft = SourceMetadataDraft {
            figure_id: "fig_6".to_string(),
            original_title: Some("Old".to_string()),
            original_authors: Some("A".to_string()),
            original_published_at: None,
            title_input: "Old".to_string(),
            authors_input: "A".to_string(),
            published_at_input: "".to_string(),
            clear_title: true,
            clear_authors: true,
            clear_published_at: true,
            lifecycle: SourceEditorLifecycle::EditingDirty,
            last_error: None,
        };

        let payload = build_source_update_payload(&draft);

        assert!(payload.has_changes);
        assert!(payload.clear_title);
        assert!(payload.clear_authors);
        assert!(payload.clear_published_at);
        assert_eq!(payload.title, None);
        assert_eq!(payload.authors, None);
        assert_eq!(payload.published_at, None);
    }

    #[test]
    fn sync_figure_lifecycle_transitions_clean_dirty_and_preserves_saving() {
        let mut draft = FigureMetadataDraft {
            figure_id: "fig_7".to_string(),
            original_display_name: "original".to_string(),
            original_caption: None,
            display_name_input: "original".to_string(),
            caption_input: String::new(),
            clear_caption: false,
            lifecycle: FigureEditorLifecycle::EditingClean,
            last_error: None,
        };

        sync_figure_draft_lifecycle(&mut draft, true);
        assert_eq!(draft.lifecycle, FigureEditorLifecycle::EditingDirty);

        sync_figure_draft_lifecycle(&mut draft, false);
        assert_eq!(draft.lifecycle, FigureEditorLifecycle::EditingClean);

        draft.lifecycle = FigureEditorLifecycle::Saving;
        sync_figure_draft_lifecycle(&mut draft, false);
        assert_eq!(draft.lifecycle, FigureEditorLifecycle::Saving);
    }

    #[test]
    fn sync_source_lifecycle_transitions_clean_dirty_and_preserves_saving() {
        let mut draft = SourceMetadataDraft {
            figure_id: "fig_8".to_string(),
            original_title: Some("Title".to_string()),
            original_authors: None,
            original_published_at: None,
            title_input: "Title".to_string(),
            authors_input: String::new(),
            published_at_input: String::new(),
            clear_title: false,
            clear_authors: false,
            clear_published_at: false,
            lifecycle: SourceEditorLifecycle::EditingClean,
            last_error: None,
        };

        sync_source_draft_lifecycle(&mut draft, true);
        assert_eq!(draft.lifecycle, SourceEditorLifecycle::EditingDirty);

        sync_source_draft_lifecycle(&mut draft, false);
        assert_eq!(draft.lifecycle, SourceEditorLifecycle::EditingClean);

        draft.lifecycle = SourceEditorLifecycle::Saving;
        sync_source_draft_lifecycle(&mut draft, false);
        assert_eq!(draft.lifecycle, SourceEditorLifecycle::Saving);
    }

    #[test]
    fn sync_tag_lifecycle_transitions_clean_dirty_and_preserves_saving() {
        let mut draft = TagMutationDraft {
            figure_id: "fig_9".to_string(),
            tag_input: String::new(),
            lifecycle: TagEditorLifecycle::EditingClean,
            last_error: None,
        };

        sync_tag_draft_lifecycle(&mut draft, true);
        assert_eq!(draft.lifecycle, TagEditorLifecycle::EditingDirty);

        sync_tag_draft_lifecycle(&mut draft, false);
        assert_eq!(draft.lifecycle, TagEditorLifecycle::EditingClean);

        draft.lifecycle = TagEditorLifecycle::Saving;
        sync_tag_draft_lifecycle(&mut draft, false);
        assert_eq!(draft.lifecycle, TagEditorLifecycle::Saving);
    }

    #[test]
    fn sync_link_lifecycle_transitions_clean_dirty_and_preserves_saving() {
        let mut draft = LinkMutationDraft {
            figure_id: "fig_10".to_string(),
            to_figure_id_input: String::new(),
            relation_input: "related".to_string(),
            lifecycle: LinkEditorLifecycle::EditingClean,
            last_error: None,
        };

        sync_link_draft_lifecycle(&mut draft, true);
        assert_eq!(draft.lifecycle, LinkEditorLifecycle::EditingDirty);

        sync_link_draft_lifecycle(&mut draft, false);
        assert_eq!(draft.lifecycle, LinkEditorLifecycle::EditingClean);

        draft.lifecycle = LinkEditorLifecycle::Saving;
        sync_link_draft_lifecycle(&mut draft, false);
        assert_eq!(draft.lifecycle, LinkEditorLifecycle::Saving);
    }

    #[test]
    fn normalize_drop_path_is_stable_for_equivalent_paths() {
        let first = normalize_drop_path(Path::new("alpha/./beta/file.png"));
        let second = normalize_drop_path(Path::new("alpha/beta/file.png"));
        assert_eq!(first, second);
    }

    #[test]
    fn parse_drop_source_type_input_accepts_supported_values() {
        assert!(matches!(
            parse_drop_source_type_input("doi"),
            Ok(SourceType::Doi)
        ));
        assert!(matches!(
            parse_drop_source_type_input("URL"),
            Ok(SourceType::Url)
        ));
        assert!(matches!(
            parse_drop_source_type_input("local"),
            Ok(SourceType::Local)
        ));
        assert!(matches!(
            parse_drop_source_type_input("manual"),
            Ok(SourceType::Manual)
        ));
        assert!(parse_drop_source_type_input("unsupported").is_err());
    }

    #[test]
    fn drop_ingest_session_transitions_from_drop_received_to_metadata_required() {
        let mut app = LamianGuiApp::default();
        app.begin_drop_ingest_session(vec![
            PathBuf::from("zeta/file_b.png"),
            PathBuf::from("alpha/file_a.png"),
        ]);

        assert_eq!(
            app.drop_ingest_session.lifecycle,
            DropIngestLifecycle::MetadataRequired
        );
        assert_eq!(
            app.drop_ingest_session
                .dropped_items
                .iter()
                .map(|item| item.normalized_path.as_str())
                .collect::<Vec<_>>(),
            vec!["alpha/file_a.png", "zeta/file_b.png"]
        );
        assert_eq!(
            drop_ingest_lifecycle_label(app.drop_ingest_session.lifecycle),
            "metadata_required"
        );
    }

    #[test]
    fn drop_ingest_session_reaches_ready_to_commit_after_metadata_completion() {
        let mut app = LamianGuiApp::default();
        app.begin_drop_ingest_session(vec![
            PathBuf::from("alpha/file_a.png"),
            PathBuf::from("beta/file_b.png"),
        ]);

        for item in &mut app.drop_ingest_session.dropped_items {
            item.source_type_input = "doi".to_string();
            item.source_key_input = format!("10.1000/{}", item.normalized_path);
        }
        app.sync_drop_ingest_lifecycle();

        assert_eq!(
            app.drop_ingest_session.lifecycle,
            DropIngestLifecycle::ReadyToCommit
        );
    }

    #[test]
    fn drop_ingest_commit_imports_one_or_many_files_via_shared_inject_core() {
        let temp_dir = TempDir::new().expect("temp directory");
        let vault_path = temp_dir.path().join("vault");
        db::initialize_vault(&vault_path).expect("initialize vault");

        let fixture_path = repository_fixture_path("2602.17205_1.png");
        let file_a_path = temp_dir.path().join("drop_a.png");
        let file_b_path = temp_dir.path().join("drop_b.png");
        fs::copy(&fixture_path, &file_a_path).expect("copy fixture a");
        fs::copy(&fixture_path, &file_b_path).expect("copy fixture b");

        let mut app = LamianGuiApp {
            connected_vault_root: Some(vault_path),
            ..LamianGuiApp::default()
        };
        app.refresh_figure_rows();
        assert!(app.figure_rows.is_empty());

        app.begin_drop_ingest_session(vec![file_b_path, file_a_path]);
        for (index, item) in app.drop_ingest_session.dropped_items.iter_mut().enumerate() {
            item.source_type_input = "doi".to_string();
            item.source_key_input = format!("10.1000/drop-{index}");
        }
        app.sync_drop_ingest_lifecycle();
        assert_eq!(
            app.drop_ingest_session.lifecycle,
            DropIngestLifecycle::ReadyToCommit
        );

        app.begin_drop_ingest_commit();

        assert_eq!(
            app.drop_ingest_session.lifecycle,
            DropIngestLifecycle::Committed
        );
        assert_eq!(app.drop_ingest_session.last_commit_results.len(), 2);
        assert!(app
            .drop_ingest_session
            .last_commit_results
            .iter()
            .all(|item| item.error.is_none()));
        assert_eq!(app.figure_rows.len(), 2);
    }

    #[test]
    fn drop_ingest_commit_supports_partial_failure_without_reordering_successes() {
        let temp_dir = TempDir::new().expect("temp directory");
        let vault_path = temp_dir.path().join("vault");
        db::initialize_vault(&vault_path).expect("initialize vault");

        let fixture_path = repository_fixture_path("2602.17205_1.png");
        let file_a_path = temp_dir.path().join("drop_a.png");
        let file_b_path = temp_dir.path().join("drop_b.png");
        fs::copy(&fixture_path, &file_a_path).expect("copy fixture a");
        fs::copy(&fixture_path, &file_b_path).expect("copy fixture b");

        let mut app = LamianGuiApp {
            connected_vault_root: Some(vault_path),
            ..LamianGuiApp::default()
        };
        app.begin_drop_ingest_session(vec![file_b_path, file_a_path]);
        for item in &mut app.drop_ingest_session.dropped_items {
            if item.normalized_path.ends_with("drop_a.png") {
                item.source_type_input = "doi".to_string();
                item.source_key_input = "10.1000/good".to_string();
            } else {
                item.source_type_input = "invalid".to_string();
                item.source_key_input = "10.1000/bad".to_string();
            }
        }
        app.sync_drop_ingest_lifecycle();
        assert_eq!(
            app.drop_ingest_session.lifecycle,
            DropIngestLifecycle::ReadyToCommit
        );

        app.begin_drop_ingest_commit();

        assert_eq!(
            app.drop_ingest_session.lifecycle,
            DropIngestLifecycle::CommitFailed
        );
        assert_eq!(app.drop_ingest_session.last_commit_results.len(), 2);
        let failed_count = app
            .drop_ingest_session
            .last_commit_results
            .iter()
            .filter(|item| item.error.is_some())
            .count();
        assert_eq!(failed_count, 1);
        assert_eq!(app.figure_rows.len(), 1);
    }

    #[test]
    fn figure_save_failure_keeps_draft_and_allows_retry() {
        let (_temp_dir, vault_path, mut app, figure_ids) = seed_app_with_two_figures();
        let figure_id = figure_ids[0].clone();

        app.connected_vault_root = Some(vault_path);
        app.load_figure_detail(&figure_id);
        app.begin_figure_metadata_editing();

        {
            let draft = app
                .figure_metadata_draft
                .as_mut()
                .expect("figure draft initialized");
            draft.display_name_input = "   ".to_string();
        }
        let failure_payload = build_figure_update_payload(
            app.figure_metadata_draft
                .as_ref()
                .expect("figure draft present"),
        );
        assert!(failure_payload.has_changes);

        app.save_figure_metadata_changes(failure_payload);

        let failed_draft = app
            .figure_metadata_draft
            .as_ref()
            .expect("draft persists after failed save");
        assert_eq!(failed_draft.lifecycle, FigureEditorLifecycle::SaveFailed);
        assert!(failed_draft
            .last_error
            .as_deref()
            .is_some_and(|message| message.contains("name")));
        assert!(app.error_message.is_some());

        {
            let draft = app
                .figure_metadata_draft
                .as_mut()
                .expect("figure draft still present");
            draft.display_name_input = "Retry Name".to_string();
        }
        let retry_payload = build_figure_update_payload(
            app.figure_metadata_draft
                .as_ref()
                .expect("figure draft present for retry"),
        );
        assert!(retry_payload.has_changes);

        app.save_figure_metadata_changes(retry_payload);

        assert!(app.figure_metadata_draft.is_none());
        assert!(app.error_message.is_none());
        assert_eq!(
            app.selected_figure_detail
                .as_ref()
                .map(|detail| detail.display_name.as_str()),
            Some("Retry Name")
        );
    }

    #[test]
    fn source_save_failure_keeps_draft_and_allows_retry() {
        let (_temp_dir, vault_path, mut app, figure_ids) = seed_app_with_two_figures();
        let figure_id = figure_ids[0].clone();

        update_source_metadata(UpdateSourceRequest {
            vault_root: vault_path.clone(),
            figure_id: figure_id.clone(),
            title: None,
            authors: Some("Original Authors".to_string()),
            published_at: None,
            clear_title: false,
            clear_authors: false,
            clear_published_at: false,
        })
        .expect("set initial source authors");

        app.connected_vault_root = Some(vault_path);
        app.load_figure_detail(&figure_id);
        app.begin_source_metadata_editing();

        {
            let draft = app
                .source_metadata_draft
                .as_mut()
                .expect("source draft initialized");
            draft.authors_input = "   ".to_string();
        }
        let failure_payload = build_source_update_payload(
            app.source_metadata_draft
                .as_ref()
                .expect("source draft present"),
        );
        assert!(failure_payload.has_changes);

        app.save_source_metadata_changes(failure_payload);

        let failed_draft = app
            .source_metadata_draft
            .as_ref()
            .expect("draft persists after failed save");
        assert_eq!(failed_draft.lifecycle, SourceEditorLifecycle::SaveFailed);
        assert!(failed_draft
            .last_error
            .as_deref()
            .is_some_and(|message| message.contains("authors")));
        assert!(app.error_message.is_some());

        {
            let draft = app
                .source_metadata_draft
                .as_mut()
                .expect("source draft still present");
            draft.authors_input = "Recovered Authors".to_string();
        }
        let retry_payload = build_source_update_payload(
            app.source_metadata_draft
                .as_ref()
                .expect("source draft present for retry"),
        );
        assert!(retry_payload.has_changes);

        app.save_source_metadata_changes(retry_payload);

        assert!(app.source_metadata_draft.is_none());
        assert!(app.error_message.is_none());
        assert_eq!(
            app.selected_figure_detail
                .as_ref()
                .and_then(|detail| detail.sources.first())
                .and_then(|source| source.source_authors.as_deref()),
            Some("Recovered Authors")
        );
    }

    #[test]
    fn figure_save_with_display_name_change_preserves_deterministic_list_and_detail_selection() {
        let (_temp_dir, _vault_path, mut app, figure_ids) = seed_app_with_two_figures();
        let figure_id = figure_ids[0].clone();
        let list_order_before = app
            .figure_rows
            .iter()
            .map(|row| row.figure_id.clone())
            .collect::<Vec<_>>();

        app.load_figure_detail(&figure_id);
        app.begin_figure_metadata_editing();
        {
            let draft = app
                .figure_metadata_draft
                .as_mut()
                .expect("figure draft initialized");
            draft.display_name_input = "deterministic rename".to_string();
        }
        let payload = build_figure_update_payload(
            app.figure_metadata_draft
                .as_ref()
                .expect("figure draft present"),
        );
        assert!(payload.has_changes);
        assert!(payload.display_name_changed);

        app.save_figure_metadata_changes(payload);

        let list_order_after = app
            .figure_rows
            .iter()
            .map(|row| row.figure_id.clone())
            .collect::<Vec<_>>();
        assert_eq!(list_order_after, list_order_before);
        assert_eq!(app.selected_figure_id.as_deref(), Some(figure_id.as_str()));
        assert_eq!(
            app.selected_figure_detail
                .as_ref()
                .map(|detail| detail.figure_id.as_str()),
            Some(figure_id.as_str())
        );
        assert_eq!(
            app.selected_figure_detail
                .as_ref()
                .map(|detail| detail.display_name.as_str()),
            Some("deterministic rename")
        );
    }

    #[test]
    fn source_save_preserves_deterministic_list_and_refreshes_detail() {
        let (_temp_dir, _vault_path, mut app, figure_ids) = seed_app_with_two_figures();
        let figure_id = figure_ids[0].clone();
        let list_order_before = app
            .figure_rows
            .iter()
            .map(|row| row.figure_id.clone())
            .collect::<Vec<_>>();

        app.load_figure_detail(&figure_id);
        app.begin_source_metadata_editing();
        {
            let draft = app
                .source_metadata_draft
                .as_mut()
                .expect("source draft initialized");
            draft.title_input = "Updated Source Title".to_string();
        }
        let payload = build_source_update_payload(
            app.source_metadata_draft
                .as_ref()
                .expect("source draft present"),
        );
        assert!(payload.has_changes);

        app.save_source_metadata_changes(payload);

        let list_order_after = app
            .figure_rows
            .iter()
            .map(|row| row.figure_id.clone())
            .collect::<Vec<_>>();
        assert_eq!(list_order_after, list_order_before);
        assert_eq!(app.selected_figure_id.as_deref(), Some(figure_id.as_str()));
        assert_eq!(
            app.selected_figure_detail
                .as_ref()
                .map(|detail| detail.figure_id.as_str()),
            Some(figure_id.as_str())
        );
        assert_eq!(
            app.selected_figure_detail
                .as_ref()
                .and_then(|detail| detail.sources.first())
                .and_then(|source| source.source_title.as_deref()),
            Some("Updated Source Title")
        );
    }

    #[test]
    fn tag_add_and_remove_preserve_deterministic_list_and_detail_selection() {
        let (_temp_dir, _vault_path, mut app, figure_ids) = seed_app_with_two_figures();
        let figure_id = figure_ids[0].clone();
        let list_order_before = app
            .figure_rows
            .iter()
            .map(|row| row.figure_id.clone())
            .collect::<Vec<_>>();

        app.load_figure_detail(&figure_id);
        app.begin_tag_mutation_editing();
        {
            let draft = app
                .tag_mutation_draft
                .as_mut()
                .expect("tag draft initialized");
            draft.tag_input = "topic:nebula".to_string();
        }
        let add_payload = build_tag_mutation_payload(
            app.tag_mutation_draft.as_ref().expect("tag draft present"),
            TagMutationAction::Add,
        );
        assert!(add_payload.has_changes);

        app.save_tag_mutation_changes(add_payload);
        assert!(app.tag_mutation_draft.is_none());
        assert_eq!(
            app.selected_figure_detail
                .as_ref()
                .map(|detail| detail.figure_id.as_str()),
            Some(figure_id.as_str())
        );
        assert!(app
            .selected_figure_detail
            .as_ref()
            .is_some_and(|detail| detail.tags.iter().any(|tag| tag == "topic:nebula")));

        app.begin_tag_mutation_editing();
        {
            let draft = app
                .tag_mutation_draft
                .as_mut()
                .expect("tag draft initialized for removal");
            draft.tag_input = "topic:nebula".to_string();
        }
        let remove_payload = build_tag_mutation_payload(
            app.tag_mutation_draft
                .as_ref()
                .expect("tag draft present for removal"),
            TagMutationAction::Remove,
        );
        assert!(remove_payload.has_changes);

        app.save_tag_mutation_changes(remove_payload);
        let list_order_after = app
            .figure_rows
            .iter()
            .map(|row| row.figure_id.clone())
            .collect::<Vec<_>>();
        assert_eq!(list_order_after, list_order_before);
        assert_eq!(app.selected_figure_id.as_deref(), Some(figure_id.as_str()));
        assert!(app
            .selected_figure_detail
            .as_ref()
            .is_some_and(|detail| !detail.tags.iter().any(|tag| tag == "topic:nebula")));
    }

    #[test]
    fn tag_save_failure_keeps_draft_and_allows_retry() {
        let (_temp_dir, _vault_path, mut app, figure_ids) = seed_app_with_two_figures();
        let figure_id = figure_ids[0].clone();

        app.load_figure_detail(&figure_id);
        app.begin_tag_mutation_editing();
        {
            let draft = app
                .tag_mutation_draft
                .as_mut()
                .expect("tag draft initialized");
            draft.tag_input = "bad tag".to_string();
        }
        let failure_payload = build_tag_mutation_payload(
            app.tag_mutation_draft.as_ref().expect("tag draft present"),
            TagMutationAction::Add,
        );
        assert!(failure_payload.has_changes);

        app.save_tag_mutation_changes(failure_payload);

        let failed_draft = app
            .tag_mutation_draft
            .as_ref()
            .expect("draft persists after failed save");
        assert_eq!(failed_draft.lifecycle, TagEditorLifecycle::SaveFailed);
        assert!(failed_draft
            .last_error
            .as_deref()
            .is_some_and(|message| message.contains("invalid tag")));

        {
            let draft = app
                .tag_mutation_draft
                .as_mut()
                .expect("tag draft still present");
            draft.tag_input = "topic:retry".to_string();
        }
        let retry_payload = build_tag_mutation_payload(
            app.tag_mutation_draft
                .as_ref()
                .expect("tag draft present for retry"),
            TagMutationAction::Add,
        );
        assert!(retry_payload.has_changes);

        app.save_tag_mutation_changes(retry_payload);

        assert!(app.tag_mutation_draft.is_none());
        assert!(app.error_message.is_none());
        assert!(app
            .selected_figure_detail
            .as_ref()
            .is_some_and(|detail| detail.tags.iter().any(|tag| tag == "topic:retry")));
    }

    #[test]
    fn link_add_and_remove_preserve_deterministic_list_and_detail_selection() {
        let (_temp_dir, _vault_path, mut app, figure_ids) = seed_app_with_two_figures();
        let figure_id = figure_ids[0].clone();
        let target_id = figure_ids[1].clone();
        let list_order_before = app
            .figure_rows
            .iter()
            .map(|row| row.figure_id.clone())
            .collect::<Vec<_>>();

        app.load_figure_detail(&figure_id);
        app.begin_link_mutation_editing();
        {
            let draft = app
                .link_mutation_draft
                .as_mut()
                .expect("link draft initialized");
            draft.to_figure_id_input = target_id.clone();
            draft.relation_input = "cites".to_string();
        }
        let add_payload = build_link_mutation_payload(
            app.link_mutation_draft
                .as_ref()
                .expect("link draft present"),
            LinkMutationAction::Add,
        );
        assert!(add_payload.has_changes);

        app.save_link_mutation_changes(add_payload);
        assert!(app.link_mutation_draft.is_none());
        assert_eq!(app.selected_figure_id.as_deref(), Some(figure_id.as_str()));
        assert!(app.selected_figure_detail.as_ref().is_some_and(|detail| {
            detail
                .outbound_links
                .iter()
                .any(|link| link.to_figure_id == target_id && link.relation_type == "cites")
        }));

        app.begin_link_mutation_editing();
        {
            let draft = app
                .link_mutation_draft
                .as_mut()
                .expect("link draft initialized for removal");
            draft.to_figure_id_input = target_id.clone();
            draft.relation_input = "related".to_string();
        }
        let remove_payload = build_link_mutation_payload(
            app.link_mutation_draft
                .as_ref()
                .expect("link draft present for removal"),
            LinkMutationAction::Remove,
        );
        assert!(remove_payload.has_changes);

        app.save_link_mutation_changes(remove_payload);
        let list_order_after = app
            .figure_rows
            .iter()
            .map(|row| row.figure_id.clone())
            .collect::<Vec<_>>();
        assert_eq!(list_order_after, list_order_before);
        assert_eq!(app.selected_figure_id.as_deref(), Some(figure_id.as_str()));
        assert!(app.selected_figure_detail.as_ref().is_some_and(|detail| {
            detail
                .outbound_links
                .iter()
                .all(|link| link.to_figure_id != target_id)
        }));
    }

    #[test]
    fn link_save_failure_keeps_draft_and_allows_retry() {
        let (_temp_dir, _vault_path, mut app, figure_ids) = seed_app_with_two_figures();
        let figure_id = figure_ids[0].clone();
        let target_id = figure_ids[1].clone();

        app.load_figure_detail(&figure_id);
        app.begin_link_mutation_editing();
        {
            let draft = app
                .link_mutation_draft
                .as_mut()
                .expect("link draft initialized");
            draft.to_figure_id_input = figure_id.clone();
            draft.relation_input = "related".to_string();
        }
        let failure_payload = build_link_mutation_payload(
            app.link_mutation_draft
                .as_ref()
                .expect("link draft present"),
            LinkMutationAction::Add,
        );
        assert!(failure_payload.has_changes);

        app.save_link_mutation_changes(failure_payload);

        let failed_draft = app
            .link_mutation_draft
            .as_ref()
            .expect("draft persists after failed save");
        assert_eq!(failed_draft.lifecycle, LinkEditorLifecycle::SaveFailed);
        assert!(failed_draft
            .last_error
            .as_deref()
            .is_some_and(|message| message.contains("self-link")));

        {
            let draft = app
                .link_mutation_draft
                .as_mut()
                .expect("link draft still present");
            draft.to_figure_id_input = target_id.clone();
            draft.relation_input = "supports".to_string();
        }
        let retry_payload = build_link_mutation_payload(
            app.link_mutation_draft
                .as_ref()
                .expect("link draft present for retry"),
            LinkMutationAction::Add,
        );
        assert!(retry_payload.has_changes);

        app.save_link_mutation_changes(retry_payload);

        assert!(app.link_mutation_draft.is_none());
        assert!(app.error_message.is_none());
        assert!(app.selected_figure_detail.as_ref().is_some_and(|detail| {
            detail
                .outbound_links
                .iter()
                .any(|link| link.to_figure_id == target_id && link.relation_type == "supports")
        }));
    }

    #[test]
    fn delete_flow_requires_confirmation_and_uses_deterministic_next_selection() {
        let (_temp_dir, _vault_path, mut app, figure_ids) = seed_app_with_two_figures();
        let first_figure_id = figure_ids[0].clone();
        let second_figure_id = figure_ids[1].clone();

        app.load_figure_detail(&first_figure_id);
        app.begin_delete_figure_confirmation();

        let delete_draft = app
            .delete_figure_draft
            .as_ref()
            .expect("delete draft initialized");
        assert_eq!(
            delete_draft.lifecycle,
            DeleteEditorLifecycle::ConfirmingDelete
        );
        assert_eq!(delete_draft.figure_id, first_figure_id);

        app.confirm_delete_figure(DeleteFigurePayload {
            figure_id: first_figure_id.clone(),
        });

        assert!(app.delete_figure_draft.is_none());
        assert_eq!(
            app.selected_figure_id.as_deref(),
            Some(second_figure_id.as_str())
        );
        assert_eq!(app.figure_rows.len(), 1);

        app.begin_delete_figure_confirmation();
        app.confirm_delete_figure(DeleteFigurePayload {
            figure_id: second_figure_id.clone(),
        });

        assert!(app.delete_figure_draft.is_none());
        assert!(app.selected_figure_id.is_none());
        assert!(app.selected_figure_detail.is_none());
        assert!(app.figure_rows.is_empty());
    }

    #[test]
    fn delete_failure_keeps_confirmation_state_and_allows_cancel() {
        let (_temp_dir, vault_path, mut app, figure_ids) = seed_app_with_two_figures();
        let figure_id = figure_ids[0].clone();

        app.load_figure_detail(&figure_id);
        app.begin_delete_figure_confirmation();

        delete_figure(DeleteFigureRequest {
            vault_root: vault_path,
            figure_id: figure_id.clone(),
        })
        .expect("external delete to induce GUI failure path");

        app.confirm_delete_figure(DeleteFigurePayload {
            figure_id: figure_id.clone(),
        });

        let failed_draft = app
            .delete_figure_draft
            .as_ref()
            .expect("delete draft persists after failed delete");
        assert_eq!(failed_draft.lifecycle, DeleteEditorLifecycle::DeleteFailed);
        assert!(failed_draft
            .last_error
            .as_deref()
            .is_some_and(|message| message.contains("unknown figure id")));
        assert!(app.error_message.is_some());

        app.cancel_delete_figure_confirmation();
        assert!(app.delete_figure_draft.is_none());
    }

    fn seed_app_with_two_figures() -> (TempDir, PathBuf, LamianGuiApp, Vec<String>) {
        let temp_dir = TempDir::new().expect("temp directory");
        let vault_path = temp_dir.path().join("vault");
        db::initialize_vault(&vault_path).expect("initialize vault");

        let fixture_path = repository_fixture_path("2602.17205_1.png");
        let first_figure = inject_figure(InjectRequest {
            vault_root: vault_path.clone(),
            file_path: fixture_path.clone(),
            source_type: SourceType::Doi,
            source_key: "10.1126/science.ady9404".to_string(),
            copy_mode: CopyMode::Reference,
        })
        .expect("inject first figure");

        let second_figure = inject_figure(InjectRequest {
            vault_root: vault_path.clone(),
            file_path: fixture_path,
            source_type: SourceType::Doi,
            source_key: "10.1126/science.ady9405".to_string(),
            copy_mode: CopyMode::Reference,
        })
        .expect("inject second figure");

        let mut app = LamianGuiApp {
            connected_vault_root: Some(vault_path.clone()),
            ..LamianGuiApp::default()
        };
        app.refresh_figure_rows();

        let mut figure_ids = vec![first_figure.figure_id, second_figure.figure_id];
        figure_ids.sort();
        assert_eq!(
            app.figure_rows
                .iter()
                .map(|row| row.figure_id.clone())
                .collect::<Vec<_>>(),
            figure_ids
        );

        (temp_dir, vault_path, app, figure_ids)
    }

    fn repository_fixture_path(file_name: &str) -> PathBuf {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join(file_name);

        assert!(path.exists(), "missing fixture file: {}", path.display());
        canonicalize_path(&path)
    }

    fn canonicalize_path(path: &Path) -> PathBuf {
        path.canonicalize()
            .unwrap_or_else(|error| panic!("failed to canonicalize {}: {error}", path.display()))
    }
}
