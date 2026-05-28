use std::fs::Metadata;
use std::path::Path;

use crate::scanner::config::ScanConfig;
use crate::scanner::file_entry::FileKind;

pub fn classify_path(
    path: &Path,
    metadata: &Metadata,
    config: &ScanConfig,
) -> FileKind {
    if metadata.is_dir() {
        return FileKind::Directory;
    }

    let file_name = path
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_default();

    config
        .classify_file_name(&file_name)
        .unwrap_or(FileKind::Unknown)
}
