use eframe::egui;

use crate::gui::central::{search_results, text_view};
use crate::gui::state::{GuiState, WorkspaceViewer};

pub fn show(ui: &mut egui::Ui, state: &mut GuiState, index: usize) {
    match &mut state.tabs[index] {
        WorkspaceViewer::Welcome { scan_id } => {
            ui.heading("Project overview");
            ui.label(format!("Scan: {:?}", scan_id));
        }

        WorkspaceViewer::Bucket { bucket_index } => {
            ui.heading("Bucket files");
            ui.label(format!("Bucket index: {bucket_index}"));
        }

        WorkspaceViewer::Search { query, hits } => {
            search_results::show(ui, query, hits);
        }

        WorkspaceViewer::TextFile(open_file) => {
            text_view::show_file(ui, open_file);
        }
    }
}
