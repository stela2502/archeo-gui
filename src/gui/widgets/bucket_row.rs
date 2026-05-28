//! One row in the bucket table.
//!
//! The row is intentionally lightweight. It only renders bucket data and
//! updates selection/classification state in response to clicks.

use eframe::egui;

use crate::gui::colors;
use crate::gui::state::GuiState;
use crate::gui::widgets::classification_chip;
use crate::registry::models::BucketClassification;

/// Render one bucket row.
pub fn show(
    ui: &mut egui::Ui,
    state: &mut GuiState,
    index: usize,
) {
    if index >= state.buckets.len() {
        return;
    }

    let is_selected = state.selected_bucket == Some(index);
    let bucket = &mut state.buckets[index];

    let row_background = if is_selected {
        colors::SELECTED_ROW
    } else {
        ui.visuals().extreme_bg_color
    };

    egui::Frame::none()
        .fill(row_background)
        .inner_margin(egui::Margin::same(4))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let clicked = ui
                    .selectable_label(
                        is_selected,
                        format!(
                            "{} / {}",
                            bucket.file_kind,
                            bucket
                                .extension
                                .as_deref()
                                .unwrap_or("no_extension")
                        ),
                    )
                    .clicked();

                if clicked {
                    state.selected_bucket = Some(index);
                }

                ui.add_space(8.0);

                classification_chip::show_editor(
                    ui,
                    &mut bucket.classification,
                );

                ui.add_space(8.0);

                ui.label(format!("{}", bucket.len()));

                ui.add_space(8.0);

                ui.label(format!("{:.2} MB", bucket.human_size_mb()));

                ui.add_space(8.0);

                let example_preview = bucket
                    .examples
                    .iter()
                    .take(3)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ");

                ui.weak(example_preview);

                ui.add_space(8.0);

                if ui.small_button("generated").clicked() {
                    bucket.classification = BucketClassification::GeneratedOutput;
                }

                if ui.small_button("inspect").clicked() {
                    bucket.classification = BucketClassification::NeedsInspection;
                }
            });
        });
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
