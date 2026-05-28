//! Shared GUI colors.
//!
//! Centralizing colors here keeps the GUI visually consistent and allows
//! later theme customization without touching all widgets.

use eframe::egui::Color32;

use crate::registry::models::BucketClassification;

/// Neutral / not yet reviewed.
pub const UNREVIEWED: Color32 =
    Color32::from_rgb(120, 120, 120);

/// Explicitly excluded from archaeology workflows.
pub const EXCLUDED: Color32 =
    Color32::from_rgb(70, 70, 70);

/// Important raw or processed scientific data.
pub const IMPORTANT_DATA: Color32 =
    Color32::from_rgb(70, 130, 220);

/// Computational workflows, scripts, notebooks.
pub const ANALYSIS_CODE: Color32 =
    Color32::from_rgb(70, 180, 90);

/// Publications, presentations, documentation.
pub const PUBLICATION: Color32 =
    Color32::from_rgb(170, 90, 220);

/// Requires human inspection.
pub const NEEDS_INSPECTION: Color32 =
    Color32::from_rgb(220, 200, 70);

/// Likely reproducible/generated output.
pub const GENERATED_OUTPUT: Color32 =
    Color32::from_rgb(230, 140, 40);

/// Broken/problematic/trash/temp.
pub const PROBLEM: Color32 =
    Color32::from_rgb(210, 60, 60);

/// Background highlight for selected rows.
pub const SELECTED_ROW: Color32 =
    Color32::from_rgb(45, 45, 60);

/// Return the UI color for a bucket classification.
pub fn classification_color(
    classification: BucketClassification,
) -> Color32 {
    match classification {
        BucketClassification::Unreviewed => {
            UNREVIEWED
        }

        BucketClassification::Excluded => {
            EXCLUDED
        }

        BucketClassification::ImportantData => {
            IMPORTANT_DATA
        }

        BucketClassification::AnalysisCode => {
            ANALYSIS_CODE
        }

        BucketClassification::Publication => {
            PUBLICATION
        }

        BucketClassification::NeedsInspection => {
            NEEDS_INSPECTION
        }

        BucketClassification::GeneratedOutput => {
            GENERATED_OUTPUT
        }

        BucketClassification::Problematic => {
            PROBLEM
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_classifications_have_distinct_colors() {
        let colors = vec![
            classification_color(
                BucketClassification::Unreviewed,
            ),
            classification_color(
                BucketClassification::Excluded,
            ),
            classification_color(
                BucketClassification::ImportantData,
            ),
            classification_color(
                BucketClassification::AnalysisCode,
            ),
            classification_color(
                BucketClassification::Publication,
            ),
            classification_color(
                BucketClassification::NeedsInspection,
            ),
            classification_color(
                BucketClassification::GeneratedOutput,
            ),
            classification_color(
                BucketClassification::Problematic,
            ),
        ];

        for (i, left) in colors.iter().enumerate() {
            for right in colors.iter().skip(i + 1) {
                assert_ne!(left, right);
            }
        }
    }

    #[test]
    fn selected_row_is_not_pure_black() {
        assert_ne!(
            SELECTED_ROW,
            Color32::BLACK
        );
    }
}
