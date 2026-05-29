pub mod search_results;
pub mod tab_viewer;
pub mod text_view;
pub mod open_text_file;

use eframe::egui;

use crate::gui::state::GuiState;

pub fn show(ui: &mut egui::Ui, state: &mut GuiState) {
    if state.tabs.is_empty() {
        ui.heading("Welcome");
        ui.label("No workspace tabs open yet.");
        return;
    }

    draw_tab_bar(ui, state);

    ui.separator();

    let Some(index) = state.active_tab else {
        ui.weak("No active tab.");
        return;
    };

    if index >= state.tabs.len() {
        state.active_tab = None;
        return;
    }

    tab_viewer::show(ui, state, index);
}

fn draw_tab_bar(ui: &mut egui::Ui, state: &mut GuiState) {
    let mut close_index = None;

    ui.horizontal_wrapped(|ui| {
        for index in 0..state.tabs.len() {
            let selected = state.active_tab == Some(index);
            let label = state.tabs[index].tab_label();

            if ui.selectable_label(selected, label).clicked() {
                state.active_tab = Some(index);
            }

            if ui.small_button("×").clicked() {
                close_index = Some(index);
            }
        }
    });

    if let Some(index) = close_index {
        state.tabs.remove(index);

        state.active_tab = if state.tabs.is_empty() {
            None
        } else {
            Some(index.min(state.tabs.len() - 1))
        };
    }
}