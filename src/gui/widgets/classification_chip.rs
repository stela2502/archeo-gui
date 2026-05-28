//! Small classification editor widget.
//!
//! This widget is intentionally compact because it will appear many times
//! inside bucket tables.

use eframe::egui;

use crate::gui::colors;
use crate::registry::models::BucketClassification;

/// Render a compact classification badge.
///
/// This is a read-only variant.
pub fn show(
    ui: &mut egui::Ui,
    classification: BucketClassification,
) {
    let color =
        colors::classification_color(
            classification,
        );

    ui.colored_label(
        color,
        classification.to_string(),
    );
}

/// Render an editable classification selector.
///
/// This variant allows changing bucket classifications directly inside
/// tables and detail panels.
pub fn show_editor(
    ui: &mut egui::Ui,
    classification: &mut BucketClassification,
) {
    let color =
        colors::classification_color(
            *classification,
        );

    egui::ComboBox::from_id_source(
        ui.next_auto_id(),
    )
    .selected_text(
        egui::RichText::new(
            classification.to_string(),
        )
        .color(color),
    )
    .show_ui(ui, |ui| {
        selectable(
            ui,
            classification,
            BucketClassification::Unreviewed,
        );

        selectable(
            ui,
            classification,
            BucketClassification::Excluded,
        );

        selectable(
            ui,
            classification,
            BucketClassification::ImportantData,
        );

        selectable(
            ui,
            classification,
            BucketClassification::AnalysisCode,
        );

        selectable(
            ui,
            classification,
            BucketClassification::Publication,
        );

        selectable(
            ui,
            classification,
            BucketClassification::NeedsInspection,
        );

        selectable(
            ui,
            classification,
            BucketClassification::GeneratedOutput,
        );

        selectable(
            ui,
            classification,
            BucketClassification::Problematic,
        );
    });
}

/// One selectable classification entry.
fn selectable(
    ui: &mut egui::Ui,
    current: &mut BucketClassification,
    candidate: BucketClassification,
) {
    let color =
        colors::classification_color(
            candidate,
        );

    ui.selectable_value(
        current,
        candidate,
        egui::RichText::new(
            candidate.to_string(),
        )
        .color(color),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifications_have_non_empty_names() {
        let values = vec![
            BucketClassification::Unreviewed,
            BucketClassification::Excluded,
            BucketClassification::ImportantData,
            BucketClassification::AnalysisCode,
            BucketClassification::Publication,
            BucketClassification::NeedsInspection,
            BucketClassification::GeneratedOutput,
            BucketClassification::Problematic,
        ];

        for value in values {
            assert!(
                !value.to_string().is_empty()
            );
        }
    }

    #[test]
    fn classification_colors_are_defined() {
        let color =
            colors::classification_color(
                BucketClassification::ImportantData,
            );

        assert_ne!(
            color,
            egui::Color32::TRANSPARENT
        );
    }
}
