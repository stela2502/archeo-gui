//! Left-side navigation and bucket overview panel.
//!
//! This panel is intentionally lightweight and purely state-driven.
//! Heavy work must be triggered elsewhere.

use eframe::egui;

use crate::gui::bucket_table;
use crate::gui::colors;
use crate::gui::state::{BucketSortKey, GuiState, SortDirection};
use crate::gui::widgets::bucket_row;

use crate::registry::models::FileBucket;

/// Render the left navigation panel.
pub fn show(
    ui: &mut egui::Ui,
    state: &mut GuiState,
) {
    ui.heading("Buckets");

    ui.add_space(6.0);

    show_project_info(ui, state);

    ui.separator();

    show_bucket_summary(ui, state);

    ui.separator();

    show_bucket_filters(ui, state);

    ui.separator();

    draw_sort_header(ui, state);

    // sort the indexes
    let mut indices: Vec<usize> = (0..state.buckets.len()).collect();

    indices.sort_by(|&a, &b| {
        let left = &state.buckets[a];
        let right = &state.buckets[b];

        let ord = match state.bucket_sort_key {
            BucketSortKey::Classification => {
                format!("{:?}", left.classification)
                    .cmp(&format!("{:?}", right.classification))
            }
            BucketSortKey::KindExt => {
                (&left.file_kind, &left.extension)
                    .cmp(&(&right.file_kind, &right.extension))
            }
            BucketSortKey::Count => {
                left.count.cmp(&right.count)
            }
            BucketSortKey::Size => {
                left.total_size_bytes.cmp(&right.total_size_bytes)
            }
        };

        match state.bucket_sort_direction {
            SortDirection::Asc => ord,
            SortDirection::Desc => ord.reverse(),
        }
    });

    show_bucket_list(ui, state, indices);
}


fn draw_sort_header(ui: &mut egui::Ui, state: &mut GuiState) {
    ui.horizontal_wrapped(|ui| {
        draw_sort_button(ui, state, BucketSortKey::Classification, "Class");
        draw_sort_button(ui, state, BucketSortKey::KindExt, "Kind/ext");
        draw_sort_button(ui, state, BucketSortKey::Count, "Count");
        draw_sort_button(ui, state, BucketSortKey::Size, "Size");
    });

    ui.separator();
}

fn draw_sort_button(
    ui: &mut egui::Ui,
    state: &mut GuiState,
    column: BucketSortKey,
    label: &str,
) {
    let suffix = if state.bucket_sort_key == column {
        match state.bucket_sort_direction {
            SortDirection::Asc => " ↑",
            SortDirection::Desc => " ↓",
        }
    } else {
        ""
    };

    if ui.button(format!("{label}{suffix}")).clicked() {
        if state.bucket_sort_key == column {
            state.bucket_sort_direction = match state.bucket_sort_direction {
                SortDirection::Asc => SortDirection::Desc,
                SortDirection::Desc => SortDirection::Asc,
            };
        } else {
            state.bucket_sort_key = column;
            state.bucket_sort_direction = SortDirection::Asc;
        }
    }
}

/// Show currently loaded project information.
fn show_project_info(
    ui: &mut egui::Ui,
    state: &GuiState,
) {
    if let Some(root) = &state.current_root {
        ui.label(format!(
            "Project:\n{}",
            root.display()
        ));
    } else {
        ui.weak("No project loaded");
    }

    if let Some(db) = &state.database_path {
        ui.small(format!(
            "Database:\n{}",
            db.display()
        ));
    }
}

/// Compact statistics section.
fn show_bucket_summary(
    ui: &mut egui::Ui,
    state: &GuiState,
) {
    let total_files: usize = state
        .buckets
        .iter()
        .map(|bucket| bucket.len())
        .sum();

    ui.collapsing("Summary", |ui| {
        ui.label(format!(
            "Buckets: {}",
            state.buckets.len()
        ));

        ui.label(format!(
            "Files represented: {}",
            total_files
        ));
    });
}

/// Placeholder bucket filtering UI.
///
/// Real filtering logic can later move into dedicated GUI state.
fn show_bucket_filters(
    ui: &mut egui::Ui,
    state: &mut GuiState,
) {
    ui.collapsing("Filters", |ui| {
        ui.checkbox(
            &mut state.show_excluded,
            "Show excluded",
        );

        ui.checkbox(
            &mut state.show_generated_output,
            "Show generated output",
        );

        ui.checkbox(
            &mut state.show_unreviewed_only,
            "Only unreviewed",
        );
    });
}

/// Show compact selectable bucket list.
///
/// This is not the full bucket table view.
/// It is a lightweight navigator.
fn show_bucket_list(
    ui: &mut egui::Ui,
    state: &mut GuiState,
    index: Vec<usize>,
) {
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for idx in
                index.iter()
            {
                let is_selected =
                    state.selected_bucket
                        == Some(*idx);
                let bucket = &state.buckets[*idx];

                let color =
                    colors::classification_color(
                        bucket.classification,
                    );

                let label = format!(
                    "{} / {} ({})",
                    bucket.file_kind,
                    bucket
                        .extension
                        .as_deref()
                        .unwrap_or(
                            "no_extension"
                        ),
                    bucket.len()
                );

                let response = ui.selectable_label(
                    is_selected,
                    egui::RichText::new(label)
                        .color(color),
                );

                if response.clicked() {
                    state.selected_bucket =
                        Some(*idx);
                }
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selected_bucket_logic_works() {
        let mut state =
            GuiState::default();

        state.selected_bucket = Some(3);

        assert_eq!(
            state.selected_bucket,
            Some(3)
        );
    }

    #[test]
    fn default_state_has_no_selection() {
        let state =
            GuiState::default();

        assert!(state.selected_bucket.is_none());
    }

    #[test]
    fn total_file_count_can_be_computed() {
        let state =
            GuiState::default();

        let total: usize = state
            .buckets
            .iter()
            .map(|bucket| bucket.len())
            .sum();

        assert_eq!(total, 0);
    }
}
