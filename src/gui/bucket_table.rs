//! Bucket table view.
//!
//! This module renders the central triage table of file buckets.
//! It must stay cheap: no scanning, no database writes, no AI calls here.
//! It only reads/modifies [`GuiState`] in response to user interaction.

use eframe::egui;

use crate::gui::state::GuiState;
use crate::gui::widgets::bucket_row;

/// Render the bucket table.
///
/// This function is called every egui frame, so keep it lightweight.
pub fn show(
    ui: &mut egui::Ui,
    state: &mut GuiState,
) {
    ui.heading("Artifact buckets");

    ui.label(
        "Classify buckets first. Later discussions and AI actions will use these classifications.",
    );

    ui.add_space(8.0);

    if state.buckets.is_empty() {
        ui.info("No buckets loaded yet. Select a folder and run a scan.");
        return;
    }

    draw_summary(ui, state);

    ui.separator();

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            draw_header(ui);

            for index in 0..state.buckets.len() {
                bucket_row::show(ui, state, index);
            }
        });
}

/// Draw a compact summary above the bucket table.
fn draw_summary(
    ui: &mut egui::Ui,
    state: &GuiState,
) {
    let total_files: usize = state
        .buckets
        .iter()
        .map(|bucket| bucket.len())
        .sum();

    ui.horizontal(|ui| {
        ui.label(format!("Buckets: {}", state.buckets.len()));
        ui.separator();
        ui.label(format!("Files represented: {total_files}"));

        if let Some(selected) = state.selected_bucket {
            ui.separator();
            ui.label(format!("Selected bucket: {}", selected + 1));
        }
    });
}

/// Draw the table header.
///
/// This is intentionally simple for now. If sorting/filtering becomes more
/// complex, this can evolve into a proper table widget.
fn draw_header(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.strong("Class");
        ui.add_space(110.0);
        ui.strong("Kind/ext");
        ui.add_space(110.0);
        ui.strong("Count");
        ui.add_space(40.0);
        ui.strong("Size MB");
        ui.add_space(55.0);
        ui.strong("Examples");
    });

    ui.separator();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::models::FileBucket;

    #[test]
    fn total_files_can_be_computed_from_buckets() {
        let mut state = GuiState::default();

        state.buckets = vec![
            FileBucket {
                file_kind: "figure".to_string(),
                extension: Some("png".to_string()),
                count: 10,
                total_size_bytes: 100,
                examples: vec![],
            },
            FileBucket {
                file_kind: "notebook".to_string(),
                extension: Some("ipynb".to_string()),
                count: 2,
                total_size_bytes: 200,
                examples: vec![],
            },
        ];

        let total: usize = state
            .buckets
            .iter()
            .map(|bucket| bucket.len())
            .sum();

        assert_eq!(total, 12);
    }

    #[test]
    fn empty_state_has_no_buckets() {
        let state = GuiState::default();

        assert!(state.buckets.is_empty());
    }
}
