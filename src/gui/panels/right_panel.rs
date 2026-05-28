//! Right-side detail panel.
//!
//! This panel shows details for the currently selected bucket.
//! It should remain cheap: no file loading, no database scans, no AI calls.

use eframe::egui;

use crate::gui::state::GuiState;
use crate::gui::widgets::{
    classification_chip,
    file_preview,
};

/// Render the right detail panel.
pub fn show(
    ui: &mut egui::Ui,
    state: &mut GuiState,
) {
    ui.heading("Bucket details");

    ui.add_space(6.0);

    let Some(index) = state.selected_bucket else {
        ui.weak("Select a bucket to inspect it.");
        return;
    };

    if index >= state.buckets.len() {
        ui.weak("Selected bucket no longer exists.");
        state.selected_bucket = None;
        return;
    }

    let bucket = &mut state.buckets[index];

    ui.label(format!(
        "Kind: {}",
        bucket.file_kind
    ));

    ui.label(format!(
        "Extension: {}",
        bucket
            .extension
            .as_deref()
            .unwrap_or("no_extension")
    ));

    ui.label(format!(
        "Files: {}",
        bucket.len()
    ));

    ui.label(format!(
        "Total size: {:.2} MB",
        bucket.human_size_mb()
    ));

    ui.separator();

    classification_chip::show_editor(
        ui,
        &mut bucket.classification,
    );

    ui.separator();

    ui.heading("Examples");

    file_preview::show_examples(
        ui,
        &bucket.examples,
    );

    ui.separator();

    ui.heading("Notes");

    ui.text_edit_multiline(
        &mut bucket.user_note,
    );

    ui.add_space(8.0);

    ui.horizontal(|ui| {
        if ui.button("Mark generated").clicked() {
            bucket.classification =
                crate::registry::models::BucketClassification::GeneratedOutput;
        }

        if ui.button("Needs inspection").clicked() {
            bucket.classification =
                crate::registry::models::BucketClassification::NeedsInspection;
        }
    });

    ui.horizontal(|ui| {
        if ui.button("Important data").clicked() {
            bucket.classification =
                crate::registry::models::BucketClassification::ImportantData;
        }

        if ui.button("Problem").clicked() {
            bucket.classification =
                crate::registry::models::BucketClassification::Problematic;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::models::{
        BucketClassification,
        FileBucket,
    };

    #[test]
    fn invalid_selection_can_be_detected() {
        let mut state = GuiState::default();
        state.selected_bucket = Some(5);

        assert!(state.selected_bucket.unwrap() >= state.buckets.len());
    }

    #[test]
    fn selected_bucket_can_be_accessed() {
        let mut state = GuiState::default();

        state.buckets.push(FileBucket {
            file_kind: "figure".to_string(),
            extension: Some("png".to_string()),
            count: 10,
            total_size_bytes: 1024,
            examples: vec![
                "plot.png".to_string(),
            ],
            classification: BucketClassification::Unreviewed,
            user_note: String::new(),
            ai_note: None,
        });

        state.selected_bucket = Some(0);

        let bucket = &state.buckets[state.selected_bucket.unwrap()];

        assert_eq!(bucket.file_kind, "figure");
        assert_eq!(bucket.extension.as_deref(), Some("png"));
    }
}
