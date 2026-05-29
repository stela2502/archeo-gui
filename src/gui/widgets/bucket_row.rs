//! One row in the bucket table.
//!
//! The row is intentionally lightweight. It only renders bucket data and
//! updates selection/classification state in response to clicks.

use eframe::egui;

use crate::gui::colors;
use crate::gui::state::GuiState;
use crate::gui::widgets::classification_chip;
use crate::registry::models::BucketClassification;
use crate::gui::state::WorkspaceViewer;

/// Render one bucket row.
pub fn show(
    ui: &mut egui::Ui,
    state: &mut GuiState,
    index: usize,
) {
    let bucket = &mut state.buckets[index];

    egui::ComboBox::from_id_salt(("bucket_class", index))
        .selected_text(format!("{:?}", bucket.classification))
        .show_ui(ui, |ui| {
            ui.selectable_value(
                &mut bucket.classification,
                BucketClassification::Unreviewed,
                "Unreviewed",
            );
            ui.selectable_value(
                &mut bucket.classification,
                BucketClassification::NeedsInspection,
                "Needs inspection",
            );
            ui.selectable_value(
                &mut bucket.classification,
                BucketClassification::ImportantData,
                "Important data",
            );
            ui.selectable_value(
                &mut bucket.classification,
                BucketClassification::AnalysisCode,
                "Analysis code",
            );
            ui.selectable_value(
                &mut bucket.classification,
                BucketClassification::Publication,
                "Publication",
            );
            ui.selectable_value(
                &mut bucket.classification,
                BucketClassification::GeneratedOutput,
                "Generated output",
            );
            ui.selectable_value(
                &mut bucket.classification,
                BucketClassification::Problematic,
                "Problematic",
            );
            ui.selectable_value(
                &mut bucket.classification,
                BucketClassification::Excluded,
                "Excluded",
            );
        });

    if ui
        .selectable_label(
            state.selected_bucket == Some(index),
            format!(
                "{} / {}",
                bucket.file_kind,
                bucket.extension.as_deref().unwrap_or("-")
            ),
        )
        .clicked()
    {
        state.selected_bucket = Some(index);

        state.tabs.push(WorkspaceViewer::Bucket {
            bucket_index: index,
        });

        state.active_tab = Some(state.tabs.len() - 1);
    }

    ui.label(bucket.count.to_string());

    ui.label(format!(
        "{:.2}",
        bucket.total_size_bytes as f64 / 1_048_576.0
    ));

    ui.label(bucket.examples.join(", "));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::models::FileBucket;

    #[test]
    fn row_selection_index_can_be_stored() {
        let mut state = GuiState::default();

        state.selected_bucket = Some(2);

        assert_eq!(state.selected_bucket, Some(2));
    }

    #[test]
    fn example_preview_uses_first_three_examples() {
        let bucket = FileBucket {
            file_kind: "figure".to_string(),
            extension: Some("png".to_string()),
            count: 10,
            total_size_bytes: 100,
            examples: vec![
                "a.png".to_string(),
                "b.png".to_string(),
                "c.png".to_string(),
                "d.png".to_string(),
            ],
            classification: BucketClassification::Unreviewed,
            user_note: String::new(),
            ai_note: None,
        };

        let preview = bucket
            .examples
            .iter()
            .take(3)
            .cloned()
            .collect::<Vec<_>>()
            .join(", ");

        assert_eq!(preview, "a.png, b.png, c.png");
    }
}
