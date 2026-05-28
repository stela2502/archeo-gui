//! Top status bar.
//!
//! This panel provides lightweight runtime information and global actions.
//! It is rendered every frame and must stay cheap.

use eframe::egui;

use crate::gui::state::GuiState;

/// Render the global status bar.
pub fn show(
    ui: &mut egui::Ui,
    state: &mut GuiState,
) {
    ui.horizontal_wrapped(|ui| {
        show_project_status(ui, state);

        ui.separator();

        show_bucket_status(ui, state);

        ui.separator();

        show_runtime_flags(ui, state);

        ui.separator();

        show_action_buttons(ui, state);
    });
}

/// Show currently loaded project information.
fn show_project_status(
    ui: &mut egui::Ui,
    state: &GuiState,
) {
    if let Some(root) = &state.current_root {
        ui.label(format!(
            "Project: {}",
            root.display()
        ));
    } else {
        ui.weak("No project loaded");
    }
}

/// Show bucket statistics.
fn show_bucket_status(
    ui: &mut egui::Ui,
    state: &GuiState,
) {
    let total_files: usize = state
        .buckets
        .iter()
        .map(|bucket| bucket.len())
        .sum();

    ui.label(format!(
        "Buckets: {}",
        state.buckets.len()
    ));

    ui.separator();

    ui.label(format!(
        "Files: {}",
        total_files
    ));

    if let Some(index) = state.selected_bucket {
        ui.separator();

        ui.label(format!(
            "Selected: {}",
            index + 1
        ));
    }
}

/// Show current runtime flags.
///
/// These are simple indicators only.
/// No expensive computations should happen here.
fn show_runtime_flags(
    ui: &mut egui::Ui,
    state: &GuiState,
) {
    if state.scan_in_progress {
        ui.colored_label(
            egui::Color32::YELLOW,
            "Scanning...",
        );
    }

    if state.ai_busy {
        ui.colored_label(
            egui::Color32::LIGHT_RED,
            "AI busy",
        );
    }

    if !state.last_status_message.is_empty() {
        ui.separator();

        ui.small(
            &state.last_status_message,
        );
    }
}

/// Global actions.
///
/// These only toggle state flags.
/// Heavy operations must happen elsewhere.
fn show_action_buttons(
    ui: &mut egui::Ui,
    state: &mut GuiState,
) {
    if ui.button("Rescan").clicked() {
        state.request_rescan = true;
        state.last_status_message =
            "Rescan requested".to_string();
    }

    if ui.button("Recluster").clicked() {
        state.request_recluster = true;
        state.last_status_message =
            "Recluster requested".to_string();
    }

    if ui.button("Save").clicked() {
        state.request_save = true;
        state.last_status_message =
            "Save requested".to_string();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_state_has_no_runtime_flags() {
        let state =
            GuiState::default();

        assert!(!state.scan_in_progress);
        assert!(!state.ai_busy);
    }

    #[test]
    fn status_message_can_be_updated() {
        let mut state =
            GuiState::default();

        state.last_status_message =
            "Scanning".to_string();

        assert_eq!(
            state.last_status_message,
            "Scanning"
        );
    }

    #[test]
    fn request_flags_can_be_toggled() {
        let mut state =
            GuiState::default();

        state.request_rescan = true;

        assert!(state.request_rescan);
    }
}
