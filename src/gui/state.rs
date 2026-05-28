//! Central GUI application state.
//!
//! This module contains long-lived UI/application state.
//! Heavy computations should happen elsewhere and only update this state.

use std::path::PathBuf;

use crate::registry::models::{
    FileBucket,
    ScanRun,
};

/// Global GUI application state.
///
/// This state survives across egui frames and represents the
/// currently loaded archaeology session.
#[derive(Debug)]
pub struct GuiState {
    /// Currently loaded project root.
    pub current_root: Option<PathBuf>,

    /// Active SQLite database path.
    pub database_path: Option<PathBuf>,

    /// Currently loaded scan run.
    pub scan_run: Option<ScanRun>,

    /// Bucket summaries shown in the GUI.
    pub buckets: Vec<FileBucket>,

    /// Currently selected bucket index.
    pub selected_bucket: Option<usize>,

    /// Whether a scan is currently running.
    pub scan_in_progress: bool,

    /// Whether an AI operation is currently running.
    pub ai_busy: bool,

    /// Last lightweight status message.
    pub last_status_message: String,

    /// Request a rescan during the next update cycle.
    pub request_rescan: bool,

    /// Request reclustering during the next update cycle.
    pub request_recluster: bool,

    /// Request saving GUI/database state.
    pub request_save: bool,

    /// Show excluded buckets/files.
    pub show_excluded: bool,

    /// Show generated outputs.
    pub show_generated_output: bool,

    /// Filter to unreviewed buckets only.
    pub show_unreviewed_only: bool,
}

impl Default for GuiState {
    fn default() -> Self {
        Self {
            current_root: None,
            database_path: None,

            scan_run: None,

            buckets: Vec::new(),

            selected_bucket: None,

            scan_in_progress: false,
            ai_busy: false,

            last_status_message: String::new(),

            request_rescan: false,
            request_recluster: false,
            request_save: false,

            show_excluded: true,
            show_generated_output: true,
            show_unreviewed_only: false,
        }
    }
}

impl GuiState {
    /// Clear all currently loaded bucket data.
    ///
    /// Useful when switching projects.
    pub fn clear_buckets(&mut self) {
        self.buckets.clear();
        self.selected_bucket = None;
    }

    /// Return the currently selected bucket.
    pub fn selected_bucket(
        &self,
    ) -> Option<&FileBucket> {
        self.selected_bucket
            .and_then(|index| {
                self.buckets.get(index)
            })
    }

    /// Return the currently selected bucket mutably.
    pub fn selected_bucket_mut(
        &mut self,
    ) -> Option<&mut FileBucket> {
        self.selected_bucket
            .and_then(|index| {
                self.buckets.get_mut(index)
            })
    }

    /// Total number of files represented by buckets.
    pub fn total_files(&self) -> usize {
        self.buckets
            .iter()
            .map(|bucket| bucket.len())
            .sum()
    }

    /// Reset all transient request flags.
    pub fn clear_requests(&mut self) {
        self.request_rescan = false;
        self.request_recluster = false;
        self.request_save = false;
    }

    /// Set a lightweight status message.
    pub fn set_status(
        &mut self,
        message: impl Into<String>,
    ) {
        self.last_status_message =
            message.into();
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
    fn default_state_is_empty() {
        let state =
            GuiState::default();

        assert!(state.current_root.is_none());
        assert!(state.database_path.is_none());
        assert!(state.buckets.is_empty());
        assert!(state.selected_bucket.is_none());
    }

    #[test]
    fn total_files_is_computed_correctly() {
        let mut state =
            GuiState::default();

        state.buckets.push(FileBucket {
            file_kind: "figure".to_string(),
            extension: Some("png".to_string()),
            count: 10,
            total_size_bytes: 100,
            examples: vec![],
            classification:
                BucketClassification::Unreviewed,
            user_note: String::new(),
            ai_note: None,
        });

        state.buckets.push(FileBucket {
            file_kind: "notebook".to_string(),
            extension: Some("ipynb".to_string()),
            count: 2,
            total_size_bytes: 200,
            examples: vec![],
            classification:
                BucketClassification::Unreviewed,
            user_note: String::new(),
            ai_note: None,
        });

        assert_eq!(
            state.total_files(),
            12
        );
    }

    #[test]
    fn clear_requests_resets_flags() {
        let mut state =
            GuiState::default();

        state.request_rescan = true;
        state.request_recluster = true;
        state.request_save = true;

        state.clear_requests();

        assert!(!state.request_rescan);
        assert!(!state.request_recluster);
        assert!(!state.request_save);
    }

    #[test]
    fn selected_bucket_returns_correct_bucket() {
        let mut state =
            GuiState::default();

        state.buckets.push(FileBucket {
            file_kind: "figure".to_string(),
            extension: Some("png".to_string()),
            count: 3,
            total_size_bytes: 100,
            examples: vec![],
            classification:
                BucketClassification::Unreviewed,
            user_note: String::new(),
            ai_note: None,
        });

        state.selected_bucket = Some(0);

        let bucket =
            state.selected_bucket().unwrap();

        assert_eq!(
            bucket.file_kind,
            "figure"
        );
    }
}
