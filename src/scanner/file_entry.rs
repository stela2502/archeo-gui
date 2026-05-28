use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileEntry {
    pub id: String,
    pub scan_id: String,

    /// Path relative to the scan root.
    pub rel_path: PathBuf,

    /// Just the filename component.
    pub file_name: String,

    /// Lowercase extension without '.'.
    pub extension: Option<String>,

    pub file_kind: FileKind,

    pub size_bytes: u64,

    /// RFC3339 timestamp string for SQLite simplicity.
    pub modified_at: Option<String>,

    /// Included into the active registry.
    pub included: bool,

    /// Why the file was excluded.
    pub exclusion_reason: Option<String>,
}

impl FileEntry {
    pub fn new(
        scan_id: impl Into<String>,
        rel_path: PathBuf,
        file_kind: FileKind,
        size_bytes: u64,
    ) -> Self {
        let file_name = rel_path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();

        let extension = rel_path
            .extension()
            .map(|s| s.to_string_lossy().to_ascii_lowercase());

        Self {
            id: uuid::Uuid::new_v4().to_string(),
            scan_id: scan_id.into(),
            rel_path,
            file_name,
            extension,
            file_kind,
            size_bytes,
            modified_at: None,
            included: true,
            exclusion_reason: None,
        }
    }

    pub fn extension_str(&self) -> &str {
        self.extension.as_deref().unwrap_or("")
    }

    pub fn is_text_like(&self) -> bool {
        matches!(
            self.file_kind,
            FileKind::SourceCode
                | FileKind::Notebook
                | FileKind::Config
                | FileKind::Table
                | FileKind::Text
        )
    }

    pub fn mark_excluded(&mut self, reason: impl Into<String>) {
        self.included = false;
        self.exclusion_reason = Some(reason.into());
    }
}

#[derive(
    Debug,
    Clone,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Hash,
    Display,
    EnumString,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum FileKind {
    SourceCode,
    Notebook,
    Config,
    Table,
    Text,

    SingleCellData,
    BinaryData,
    Archive,
    Figure,
    Log,

    Directory,

    Unknown,
}


pub fn normalize_relative_path(
    root: &Path,
    path: &Path,
) -> anyhow::Result<PathBuf> {
    let rel = path.strip_prefix(root)?;

    Ok(rel.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_entry_extracts_filename_and_extension() {
        let entry = FileEntry::new(
            "scan1",
            PathBuf::from("subdir/test.IPYNB"),
            FileKind::Notebook,
            123,
        );

        assert_eq!(entry.file_name, "test.IPYNB");

        assert_eq!(
            entry.extension,

            Some("ipynb".to_string())
        );

        assert_eq!(entry.file_kind, FileKind::Notebook);

        assert_eq!(entry.size_bytes, 123);

        assert!(entry.included);
    }

    #[test]
    fn extension_str_returns_empty_for_missing_extension() {
        let entry = FileEntry::new(
            "scan1",
            PathBuf::from("README"),
            FileKind::Text,
            0,
        );

        assert_eq!(entry.extension_str(), "");
    }

    #[test]
    fn text_like_detection_works() {
        let notebook = FileEntry::new(
            "scan1",
            PathBuf::from("analysis.ipynb"),
            FileKind::Notebook,
            0,
        );

        let binary = FileEntry::new(
            "scan1",
            PathBuf::from("data.h5ad"),
            FileKind::SingleCellData,
            0,
        );

        assert!(notebook.is_text_like());

        assert!(!binary.is_text_like());
    }

    #[test]
    fn mark_excluded_updates_state() {
        let mut entry = FileEntry::new(
            "scan1",
            PathBuf::from("tmp.txt"),
            FileKind::Text,
            0,
        );

        entry.mark_excluded("temporary file");

        assert!(!entry.included);

        assert_eq!(
            entry.exclusion_reason,
            Some("temporary file".to_string())
        );
    }

    #[test]
    fn normalize_relative_path_works() {
        let root = PathBuf::from("/tmp/project");

        let file =
            PathBuf::from("/tmp/project/src/main.rs");

        let rel =
            normalize_relative_path(&root, &file).unwrap();

        assert_eq!(rel, PathBuf::from("src/main.rs"));
    }

    #[test]
    fn file_kind_display_is_stable() {
        assert_eq!(
            FileKind::Notebook.to_string(),
            "notebook"
        );

        assert_eq!(
            FileKind::SingleCellData.to_string(),
            "single_cell_data"
        );
    }
}
