//! Bucket table view.
//!
//! This module renders the central triage table of file buckets.
//! It must stay cheap: no scanning, no database writes, no AI calls here.
//! It only reads/modifies [`GuiState`] in response to user interaction.

use eframe::egui;

use crate::gui::state::GuiState;
use crate::gui::widgets::bucket_row;

use crate::registry::models::BucketClassification;

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
        match state.wizard_step {
            crate::gui::state::WizardStep::ChooseFolder => {
                ui.colored_label(
                    egui::Color32::LIGHT_GRAY,
                    "Choose a folder to begin.",
                );
            }

            crate::gui::state::WizardStep::NeedsInitialScan => {
                ui.colored_label(
                    egui::Color32::YELLOW,
                    "This folder has no scan yet. Run the initial scan.",
                );
            }

            crate::gui::state::WizardStep::Ready => {
                ui.colored_label(
                    egui::Color32::LIGHT_RED,
                    "No buckets are loaded. Try loading the existing registry or rescanning.",
                );
            }
        }

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
    let total_files = state.total_files();

    let status =
        state.project_status();

    ui.horizontal_wrapped(|ui| {
        ui.heading(status.label());

        ui.separator();

        ui.colored_label(
            status.color(),
            format!(
                "{} buckets",
                state.buckets.len()
            ),
        );

        ui.separator();

        ui.label(format!(
            "{} files",
            total_files
        ));

        ui.separator();

        let reviewed = state
            .buckets
            .iter()
            .filter(|bucket| {
                bucket.classification
                    != BucketClassification::NeedsInspection
            })
            .count();

        ui.label(format!(
            "{} reviewed",
            reviewed
        ));

        if let Some(selected) =
            state.selected_bucket
        {
            ui.separator();

            ui.label(format!(
                "Selected: {}",
                selected + 1
            ));
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
