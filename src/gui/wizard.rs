//! Folder loading wizard.
//!
//! This is the first user-facing workflow:
//!
//! 1. Choose a project folder.
//! 2. Scan it into `.archeo/archeo.sqlite`.
//! 3. Load deterministic file buckets.
//! 4. Let the user begin bucket triage.
//!
//! This module currently performs scan/load synchronously when a button is
//! clicked. That is acceptable for the first prototype because scans are fast.
//! Long AI calls should later move to a background worker.

use std::path::PathBuf;

use eframe::egui;
use mapping_info::MappingInfo;
use rfd::FileDialog;

use crate::gui::bucket_table;
use crate::gui::state::{GuiState, WizardStep};
use crate::registry::db::RegistryDb;
use crate::scanner::config::ScanConfig;
use crate::scanner::scan::scan_folder;

/// Render the central wizard/table area.
pub fn show(
    ui: &mut egui::Ui,
    state: &mut GuiState,
) {
    if state.current_root.is_none() {
        show_start_wizard(ui, state);
        return;
    }

    show_loaded_project(ui, state);

    ui.separator();

    bucket_table::show(ui, state);
}

/// Show initial folder-selection wizard.
fn show_start_wizard(
    ui: &mut egui::Ui,
    state: &mut GuiState,
) {
    ui.heading("Start archaeology session");

    ui.label(
        "Choose a research folder. archeo-gui will scan it, create a local SQLite registry, and group files into buckets.",
    );

    ui.add_space(12.0);

    if ui.button("Choose folder").clicked() {
        if let Some(folder) = FileDialog::new().pick_folder() {
            let db_path = default_db_path(&folder);

            match RegistryDb::open(&db_path) {
                Ok(db) => {
                    match db.latest_scan_run() {
                        Ok(Some(scan_run)) => {
                            state.scan_run = Some(scan_run);
                            state.wizard_step = WizardStep::Ready;

                            state.set_status(
                                "Folder selected. Existing registry found.",
                            );
                        }

                        Ok(None) => {
                            state.wizard_step =
                                WizardStep::NeedsInitialScan;

                            state.set_status(
                                "Folder selected. No scan found yet. Run initial scan.",
                            );
                        }

                        Err(err) => {
                            state.set_status(format!(
                                "Failed to inspect registry: {err}"
                            ));
                        }
                    }
                }

                Err(err) => {
                    state.set_status(format!(
                        "Failed to open/create registry: {err}"
                    ));
                }
            }
        }
    }

    if let Some(root) = &state.current_root {
        ui.separator();

        ui.label(format!("Selected: {}", root.display()));
    }
}

fn show_initial_scan_step(
    ui: &mut egui::Ui,
    state: &mut GuiState,
) {
    ui.heading("Initial scan required");

    if let Some(root) = &state.current_root {
        ui.label(format!("Folder: {}", root.display()));
    }

    ui.label("This folder has an archeo registry, but no scan run yet.");

    if ui.button("Run initial scan").clicked() {
        if let Some(root) = state.current_root.clone() {
            match scan_and_load_buckets(&root, state) {
                Ok(_) => {
                    state.wizard_step = WizardStep::Ready;
                    state.set_status("Initial scan complete.");
                }
                Err(err) => {
                    state.set_status(format!("Initial scan failed: {err}"));
                }
            }
        }
    }

    if ui.button("Choose different folder").clicked() {
        state.current_root = None;
        state.database_path = None;
        state.db = None;
        state.scan_run = None;
        state.clear_buckets();
        state.wizard_step = WizardStep::ChooseFolder;
    }
}

/// Show controls for a loaded/selected project.
fn show_loaded_project(
    ui: &mut egui::Ui,
    state: &mut GuiState,
) {
    let Some(root) = state.current_root.clone() else {
        return;
    };

    ui.horizontal(|ui| {
        ui.heading("Project");

        if ui.button("Change folder").clicked() {
            state.current_root = None;
            state.database_path = None;
            state.scan_run = None;
            state.clear_buckets();
            state.set_status("Project cleared");
        }
    });

    ui.label(root.display().to_string());

    ui.add_space(8.0);

    ui.horizontal(|ui| {
        if ui.button("Scan and load buckets").clicked() {
            if let Err(err) = scan_and_load_buckets(&root, state) {
                state.set_status(format!("Scan failed: {err}"));
            }
        }

        if ui.button("Load existing buckets").clicked() {
            if let Err(err) = load_existing_buckets(&root, state) {
                state.set_status(format!("Load failed: {err}"));
            }
        }
    });
}

/// Run scanner and then load deterministic buckets from SQLite.
///
/// This is intentionally one synchronous operation for the prototype.
/// If scans become slow, this function should be called from a worker thread.
fn scan_and_load_buckets(
    root: &PathBuf,
    state: &mut GuiState,
) -> anyhow::Result<()> {
    state.scan_in_progress = true;
    state.set_status("Scanning folder...");

    let db_path = default_db_path(root);
    let config = ScanConfig::default_profile()?;

    let mut mapping_info = MappingInfo::new(None, 1.0, 1 );

    let entries = scan_folder(root, &config, &mut mapping_info)?;

    let mut registry = RegistryDb::open(&db_path)?;

    let scan_run = registry.insert_scan_run(root, &config)?;

    registry.insert_file_entries(&scan_run.id, entries)?;

    let buckets = registry.naive_file_buckets_for_scan(
        &scan_run.id,
        3,
    )?;

    state.database_path = Some(db_path);
    state.scan_run = Some(scan_run);
    state.buckets = buckets;
    state.selected_bucket = None;

    state.scan_in_progress = false;

    state.set_status(format!("{mapping_info}"));

    Ok(())
}

/// Load latest scan and buckets from an existing database.
fn load_existing_buckets(
    root: &PathBuf,
    state: &mut GuiState,
) -> anyhow::Result<()> {
    let db_path = default_db_path(root);

    let registry = RegistryDb::open(&db_path)?;

    let Some(scan_run) =
        registry.latest_scan_run()?
    else {
        anyhow::bail!(
            "no scan run found in registry; run `archeo scan --root {}` first",
            root.display()
        );
    };

    let buckets = registry.naive_file_buckets_for_scan(
        &scan_run.id,
        3,
    )?;

    state.database_path = Some(db_path);
    state.scan_run = Some(scan_run);
    state.buckets = buckets;
    state.selected_bucket = None;

    state.set_status("Loaded existing bucket registry");

    Ok(())
}

/// Default database location for a project root.
fn default_db_path(root: &PathBuf) -> PathBuf {
    root.join(".archeo")
        .join("archeo.sqlite")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_db_path_is_inside_archeo_folder() {
        let root = PathBuf::from("/tmp/project");

        let db = default_db_path(&root);

        assert_eq!(
            db,
            PathBuf::from("/tmp/project/.archeo/archeo.sqlite")
        );
    }

    #[test]
    fn changing_folder_state_can_be_cleared() {
        let mut state = GuiState::default();

        state.current_root = Some(PathBuf::from("/tmp/project"));
        state.database_path = Some(PathBuf::from("/tmp/project/.archeo/archeo.sqlite"));

        state.current_root = None;
        state.database_path = None;
        state.clear_buckets();

        assert!(state.current_root.is_none());
        assert!(state.database_path.is_none());
        assert!(state.buckets.is_empty());
    }
}
