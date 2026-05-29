//! Right-side detail panel.
//!
//! This panel shows details for the currently selected bucket.
//! It should remain cheap: no file loading, no database scans, no AI calls.

use eframe::egui;

use crate::gui::state::{GuiState, SearchMode};
use crate::gui::widgets::{
    classification_chip,
    file_preview,
};

use crate::registry::models::BucketClassification;
use crate::gui::worker::GuiWorkerJob;
use crate::gui::state::WorkspaceViewer;
use crate::gui::central::text_view::{ open_text_file_tab };


//use crate::gui::central::search_results::SearchMode;

pub fn show(ui: &mut egui::Ui, state: &mut GuiState) {
    ui.heading("Bucket details");
    ui.add_space(6.0);

    let Some(index) = state.selected_bucket else {
        ui.weak("Select a bucket to inspect it.");
        return;
    };

    if index >= state.buckets.len() {
        ui.weak("Selected bucket no longer exists.");
        state.selected_bucket = None;
        return;
    }

    draw_bucket_summary(ui, state, index);
    ui.separator();

    draw_classification(ui, state, index);
    ui.separator();

    draw_examples(ui, state, index);
    ui.separator();

    draw_notes(ui, state, index);
    ui.separator();

    draw_search_panel(ui, state, index);
    ui.separator();

    draw_ai_panel(ui, state, index);
}

fn draw_bucket_summary(ui: &mut egui::Ui, state: &GuiState, index: usize) {
    let bucket = &state.buckets[index];

    ui.label(format!("Kind: {}", bucket.file_kind));
    ui.label(format!(
        "Extension: {}",
        bucket.extension.as_deref().unwrap_or("no_extension")
    ));
    ui.label(format!("Files: {}", bucket.len()));
    ui.label(format!("Total size: {:.2} MB", bucket.human_size_mb()));
}

fn draw_classification(ui: &mut egui::Ui, state: &mut GuiState, index: usize) {

    ui.heading("Classification");

    let bucket = &mut state.buckets[index];

    classification_chip::show_editor(ui, &mut bucket.classification);

    ui.add_space(8.0);

    ui.horizontal(|ui| {
        if ui.button("Mark generated").clicked() {
            bucket.classification = BucketClassification::GeneratedOutput;
            state.request_save = true;
        }

        if ui.button("Needs inspection").clicked() {
            bucket.classification = BucketClassification::NeedsInspection;
            state.request_save = true;
        }
    });

    ui.horizontal(|ui| {
        if ui.button("Important data").clicked() {
            bucket.classification = BucketClassification::ImportantData;
            state.request_save = true;
        }

        if ui.button("Problem").clicked() {
            bucket.classification = BucketClassification::Problematic;
            state.request_save = true;
        }
    });
}

fn draw_examples(ui: &mut egui::Ui, state: &mut GuiState, index: usize) {
    ui.heading("Examples");
    if let Some(selected_file) = file_preview::show_examples(ui, &state.buckets[index].examples){
        // open file
        open_text_file_tab( state, selected_file,None,);
    }
}

fn draw_notes(ui: &mut egui::Ui, state: &mut GuiState, index: usize) {
    ui.heading("Notes");

    let response = ui.text_edit_multiline(
        &mut state.buckets[index].user_note,
    );

    if response.changed() {
        state.request_save = true;
    }
}

fn draw_search_panel(ui: &mut egui::Ui, state: &mut GuiState, index: usize) {


    ui.heading("Find in bucket");

    ui.horizontal(|ui| {
        ui.radio_value(&mut state.search_mode, SearchMode::PlainText, "Plain text");
        ui.radio_value(&mut state.search_mode, SearchMode::Regex, "Regex");
    });

    ui.checkbox(&mut state.search_case_insensitive, "Case insensitive");

    ui.text_edit_singleline(&mut state.search_query);

    if ui.button("Find in bucket").clicked() {
        let Some(scan_run) = &state.scan_run else {
            state.set_status("No scan loaded");
            return;
        };

        let Some(db_path) = state.database_path.clone() else {
            state.set_status("No database loaded");
            return;
        };

        let bucket = state.buckets[index].clone();

        state.pending_job = Some(GuiWorkerJob::FindInBucket {
            db_path,
            scan_id: scan_run.id.clone(),
            file_kind: bucket.file_kind,
            extension: bucket.extension,
            needle: state.search_query.clone(),
            use_regex: state.search_mode == SearchMode::Regex,
            case_insensitive: state.search_case_insensitive,
        });

        state.set_status("Bucket search queued");
    }

    /*
    if !state.search_hits.is_empty() {
        ui.separator();
        ui.heading("Search hits");

        let query = state.search_query.clone();
        let hit_length = state.search_hits.len();
        let hits = std::mem::take(&mut state.search_hits);

        
        state.tabs.push(WorkspaceViewer::Search {
            query,
            hits,
        });

        state.active_tab = Some(state.tabs.len() - 1);
        state.set_status(format!("{} search hits", hit_length) );
    }
    */
}

fn draw_ai_panel(ui: &mut egui::Ui, state: &mut GuiState, index: usize) {
    use crate::gui::worker::GuiWorkerJob;

    ui.heading("Ask AI about bucket");

    ui.text_edit_multiline(&mut state.ai_question);

    if ui.button("Ask AI").clicked() {
        let bucket = &state.buckets[index];

        let prompt = format!(
            "You are helping classify a research folder bucket.\n\n\
             Bucket:\n\
             kind: {}\n\
             extension: {}\n\
             count: {}\n\
             examples:\n{}\n\n\
             User question:\n{}\n",
            bucket.file_kind,
            bucket.extension.as_deref().unwrap_or("no_extension"),
            bucket.len(),
            bucket.examples.join("\n"),
            state.ai_question
        );

        state.pending_job = Some(GuiWorkerJob::AskAi {
            prompt,
            model: state.ai_model.clone(),
        });

        state.set_status("AI question queued");
    }

    if let Some(answer) = &state.last_ai_answer {
        ui.separator();
        ui.heading("AI answer");
        ui.label(answer);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::models::{
        BucketClassification,
        FileBucket,
    };

    #[test]
    fn invalid_selection_can_be_detected() {
        let mut state = GuiState::default();
        state.selected_bucket = Some(5);

        assert!(state.selected_bucket.unwrap() >= state.buckets.len());
    }

    #[test]
    fn selected_bucket_can_be_accessed() {
        let mut state = GuiState::default();

        state.buckets.push(FileBucket {
            file_kind: "figure".to_string(),
            extension: Some("png".to_string()),
            count: 10,
            total_size_bytes: 1024,
            examples: vec![
                "plot.png".to_string(),
            ],
            classification: BucketClassification::Unreviewed,
            user_note: String::new(),
            ai_note: None,
        });

        state.selected_bucket = Some(0);

        let bucket = &state.buckets[state.selected_bucket.unwrap()];

        assert_eq!(bucket.file_kind, "figure");
        assert_eq!(bucket.extension.as_deref(), Some("png"));
    }
}
