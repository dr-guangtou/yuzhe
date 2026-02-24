use std::path::PathBuf;

use eframe::egui;

use crate::cli::{ListSortField, ListSortOrder};
use crate::list::{list_figures, ListFigureRow, ListFiguresRequest};
use crate::search::{search_figures, SearchFigure, SearchRequest};
use crate::show::{show_figure, ShowFigureRequest, ShowFigureResult};

#[derive(Debug, Clone)]
struct FigureListRowView {
    figure_id: String,
    display_name: String,
    created_at: Option<String>,
    updated_at: Option<String>,
}

#[derive(Default)]
pub struct LamianGuiApp {
    vault_root_input: String,
    connected_vault_root: Option<PathBuf>,
    search_text: String,
    figure_rows: Vec<FigureListRowView>,
    selected_figure_id: Option<String>,
    selected_figure_detail: Option<ShowFigureResult>,
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
                self.error_message = None;
            }
            Err(error) => {
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

        let Some(detail) = self.selected_figure_detail.as_ref() else {
            ui.label("Select a figure to view detail.");
            return;
        };

        ui.label(format!("Figure ID: {}", detail.figure_id));
        ui.label(format!("Display Name: {}", detail.display_name));
        ui.label(format!(
            "Caption: {}",
            detail.caption.as_deref().unwrap_or("-")
        ));
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

#[cfg(test)]
mod tests {
    use super::{figure_rows_from_list, figure_rows_from_search};
    use crate::list::ListFigureRow;
    use crate::search::SearchFigure;

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
}
