use std::path::PathBuf;

use eframe::egui;

use crate::cli::{ListSortField, ListSortOrder};
use crate::list::{list_figures, ListFigureRow, ListFiguresRequest};
use crate::search::{search_figures, SearchFigure, SearchRequest};
use crate::show::{show_figure, ShowFigureRequest, ShowFigureResult};
use crate::source::{update_source_metadata, UpdateSourceRequest};
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

        ui.label(format!("Figure ID: {}", detail.figure_id));

        let mut pending_action: Option<EditorAction> = None;
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

        if let Some(action) = pending_action {
            match action {
                EditorAction::FigureSave(payload) => self.save_figure_metadata_changes(payload),
                EditorAction::FigureCancel => self.cancel_figure_metadata_editing(),
                EditorAction::SourceSave(payload) => self.save_source_metadata_changes(payload),
                EditorAction::SourceCancel => self.cancel_source_metadata_editing(),
            }
        }

        ui.separator();
        ui.label("Outbound links:");
        for link in &detail.outbound_links {
            ui.label(format!(
                "- {} [{}] at {}",
                link.to_figure_id, link.relation_type, link.created_at
            ));
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
}

impl eframe::App for LamianGuiApp {
    fn update(&mut self, context: &egui::Context, _frame: &mut eframe::Frame) {
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

#[cfg(test)]
mod tests {
    use super::{
        build_figure_update_payload, build_source_update_payload, figure_rows_from_list,
        figure_rows_from_search, sync_figure_draft_lifecycle, sync_source_draft_lifecycle,
        FigureEditorLifecycle, FigureMetadataDraft, LamianGuiApp, SourceEditorLifecycle,
        SourceMetadataDraft,
    };
    use std::path::{Path, PathBuf};

    use tempfile::TempDir;

    use crate::db;
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
