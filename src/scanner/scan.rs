use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use walkdir::{DirEntry, WalkDir};

use crate::scanner::classify::classify_path;
use crate::scanner::config::ScanConfig;
use crate::scanner::file_entry::{normalize_relative_path, FileEntry};
use mapping_info::MappingInfo;


pub fn scan_folder(root: &Path, config: &ScanConfig, mapping_info: &mut MappingInfo ) -> Result<Vec<FileEntry>> {
    let root = root
        .canonicalize()
        .with_context(|| format!("failed to canonicalize root {}", root.display()))?;
    
    let scan_id_placeholder = "pending_scan_id";

    let mut entries = Vec::new();

    let walker = WalkDir::new(&root)
        .follow_links(config.follow_symlinks)
        .into_iter()
        .filter_entry(|entry| should_descend(entry, config));

    for entry in walker {

        let entry = match entry {
            Ok(entry) => entry,
            Err(err) => {
                mapping_info.report("walkdir_error");
                eprintln!("walkdir error: {err}");
                continue;
            }
        };

        let path = entry.path();

        if path == root {
            continue;
        }

        if entry.file_type().is_dir() {
            mapping_info.report("directories_seen");
        } else {
            mapping_info.report("files_seen");
        }

        if entry.file_type().is_symlink() {
            mapping_info.report("symlinks_seen");
        }

        let symlink_metadata = match fs::symlink_metadata(path) {
            Ok(metadata) => metadata,
            Err(err) => {
                eprintln!("Mapping error: {err:?}");
                mapping_info.report("symlinks_metadata_error");
                continue;
            }
        };

        let metadata = match fs::metadata(path) {
            Ok(metadata) => metadata,
            Err(err) => {
                if symlink_metadata.file_type().is_symlink() {
                    eprintln!("Mapping error: {err:?}");
                    mapping_info.report("broken_symlink");
                }
                continue;
            }
        };

        let rel_path = normalize_relative_path(&root, path)
            .with_context(|| format!("failed to normalize {}", path.display()))?;

        let file_kind = classify_path(path, &metadata, config);

        let mut file_entry = FileEntry::new(
            scan_id_placeholder,
            rel_path,
            file_kind,
            metadata.len(),
        );

        file_entry.modified_at = metadata.modified().ok().map(system_time_to_rfc3339);

        if should_exclude_path(path, &root, config) {
            file_entry.mark_excluded("excluded by scan profile");
            mapping_info.report("excluded_by_profile");
        } else {
            mapping_info.report("included_entries");
        }

        entries.push(file_entry);
    }

    println!("scanning report:\n{mapping_info}");

    Ok(entries)
}

fn should_descend(entry: &DirEntry, config: &ScanConfig) -> bool {
    let file_name = entry.file_name().to_string_lossy();

    if !config.include_hidden && file_name.starts_with('.') {
        return false;
    }

    if entry.file_type().is_dir() && config.dir_is_excluded(&file_name) {
        return false;
    }

    true
}

fn should_exclude_path(path: &Path, root: &Path, config: &ScanConfig) -> bool {
    let rel = path.strip_prefix(root).unwrap_or(path);

    for component in rel.components() {
        let name = component.as_os_str().to_string_lossy();

        if !config.include_hidden && name.starts_with('.') {
            return true;
        }

        if config.dir_is_excluded(&name) {
            return true;
        }
    }

    false
}

fn system_time_to_rfc3339(time: SystemTime) -> String {
    let datetime: DateTime<Utc> = time.into();
    datetime.to_rfc3339()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "archeo_gui_scan_test_{}_{}",
            name,
            uuid::Uuid::new_v4()
        ));

        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn scan_folder_registers_files() {
        let root = make_temp_dir("registers_files");

        fs::write(root.join("analysis.ipynb"), "{}").unwrap();
        fs::write(root.join("data.h5ad"), "fake").unwrap();

        let config = ScanConfig::default();
        let entries = scan_folder(&root, &config).unwrap();

        let paths: Vec<_> = entries
            .iter()
            .map(|entry| entry.rel_path.to_string_lossy().to_string())
            .collect();

        assert!(paths.contains(&"analysis.ipynb".to_string()));
        assert!(paths.contains(&"data.h5ad".to_string()));

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn scan_folder_skips_excluded_directories() {
        let root = make_temp_dir("skips_excluded_dirs");

        fs::create_dir_all(root.join("target")).unwrap();
        fs::write(root.join("target/debug.txt"), "ignore").unwrap();
        fs::write(root.join("main.rs"), "fn main() {}").unwrap();

        let config = ScanConfig::default();
        let entries = scan_folder(&root, &config).unwrap();

        let paths: Vec<_> = entries
            .iter()
            .map(|entry| entry.rel_path.to_string_lossy().to_string())
            .collect();

        assert!(paths.contains(&"main.rs".to_string()));
        assert!(!paths.contains(&"target/debug.txt".to_string()));

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn scan_folder_skips_hidden_by_default() {
        let root = make_temp_dir("skips_hidden");

        fs::write(root.join(".hidden.txt"), "ignore").unwrap();
        fs::write(root.join("visible.txt"), "keep").unwrap();

        let config = ScanConfig::default();
        let entries = scan_folder(&root, &config).unwrap();

        let paths: Vec<_> = entries
            .iter()
            .map(|entry| entry.rel_path.to_string_lossy().to_string())
            .collect();

        assert!(paths.contains(&"visible.txt".to_string()));
        assert!(!paths.contains(&".hidden.txt".to_string()));

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn scan_folder_can_include_hidden_files() {
        let root = make_temp_dir("includes_hidden");

        fs::write(root.join(".hidden.txt"), "keep").unwrap();

        let mut config = ScanConfig::default();
        config.include_hidden = true;

        let entries = scan_folder(&root, &config).unwrap();

        let paths: Vec<_> = entries
            .iter()
            .map(|entry| entry.rel_path.to_string_lossy().to_string())
            .collect();

        assert!(paths.contains(&".hidden.txt".to_string()));

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn scan_folder_sets_modified_timestamp() {
        let root = make_temp_dir("modified_timestamp");

        fs::write(root.join("visible.txt"), "keep").unwrap();

        let config = ScanConfig::default();
        let entries = scan_folder(&root, &config).unwrap();

        let entry = entries
            .iter()
            .find(|entry| entry.file_name == "visible.txt")
            .unwrap();

        assert!(entry.modified_at.is_some());

        fs::remove_dir_all(root).unwrap();
    }
}
