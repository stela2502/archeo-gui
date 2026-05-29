//! Top-level egui/eframe application.
//!
//! This file should stay small. It wires the GUI state into the frame
//! lifecycle and delegates actual UI pieces to smaller modules.

use eframe::egui;

use crate::gui::panels;
use crate::gui::state::{GuiState, WizardStep};
use crate::gui::wizard;
use crate::gui::worker::{GuiWorkerMessage, poll, dispatch};


use crate::registry::models::{ScanRun, FileBucket};

/// Main archeo-gui application.
///
/// The app owns the GUI state and implements the `eframe::App` trait.
/// All long-lived UI state should live in [`GuiState`], not as loose fields
/// in this struct.
#[derive(Debug, Default)]
pub struct ArcheoGuiApp {
    pub state: GuiState,
}



impl ArcheoGuiApp {
    /// Create a fresh GUI application.
    pub fn new() -> Self {
        Self {
            state: GuiState::default(),
        }
    }
}

impl eframe::App for ArcheoGuiApp {
    fn update(
        &mut self,
        ctx: &egui::Context,
        _frame: &mut eframe::Frame,
    ) {

        poll(&mut self.state);
        dispatch(&mut self.state);


        egui::TopBottomPanel::top("top_status_bar")
            .show(ctx, |ui| {
                panels::status_bar::show(ui, &mut self.state);
            });

        egui::SidePanel::left("left_panel")
            .resizable(true)
            .default_width(320.0)
            .show(ctx, |ui| {
                panels::left_panel::show(ui, &mut self.state);
            });

        egui::SidePanel::right("right_panel")
            .resizable(true)
            .default_width(360.0)
            .show(ctx, |ui| {
                panels::right_panel::show(ui, &mut self.state);
            });

        egui::CentralPanel::default()
            .show(ctx, |ui| {
                crate::gui::central::show(ui, &mut self.state);
            });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_can_be_created() {
        let app = ArcheoGuiApp::new();

        assert!(app.state.current_root.is_none());
        assert!(app.state.database_path.is_none());
        assert!(app.state.buckets.is_empty());
    }

    #[test]
    fn default_app_matches_new_app() {
        let app = ArcheoGuiApp::default();

        assert!(app.state.current_root.is_none());
        assert!(app.state.database_path.is_none());
        assert!(app.state.buckets.is_empty());
    }
}
