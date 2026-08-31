pub mod search_results;
pub mod tab_viewer;
pub mod text_view;
pub mod open_text_file;
pub mod markdown_viewer;

use eframe::egui;

use crate::gui::state::{GuiState, WorkspaceViewer};

pub fn show(ui: &mut egui::Ui, state: &mut GuiState) {
    if state.current_root.is_none() {
        crate::gui::wizard::show(ui, state);
        return;
    }

    ensure_landing_tab(state);

    draw_tab_bar(ui, state);

    ui.separator();

    if let Some(index) = state.active_tab {
        if index < state.tabs.len() {
            tab_viewer::show(ui, state, index);
        } else {
            state.active_tab = None;
        }
    }
}

fn ensure_landing_tab(state: &mut GuiState) {
    if state.active_tab.is_some() && !state.tabs.is_empty() {
        return;
    }

    let scan_id = state.scan_run.as_ref().map(|scan| scan.id.clone());

    state.tabs.push(WorkspaceViewer::Welcome { scan_id });
    state.active_tab = Some(state.tabs.len() - 1);
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

            ui.separator();
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