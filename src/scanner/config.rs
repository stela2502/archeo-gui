use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::scanner::file_entry::FileKind;

pub const DEFAULT_PROFILE_YAML: &str =
    include_str!("../../profiles/default.yaml");

pub const SINGLE_CELL_PROFILE_YAML: &str =
    include_str!("../../profiles/single_cell.yaml");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanConfig {
    /// Human readable profile name.
    pub name: String,

    /// Include hidden files and folders.
    #[serde(default)]
    pub include_hidden: bool,

    /// Follow symbolic links during scanning.
    #[serde(default)]
    pub follow_symlinks: bool,

    /// Maximum bytes to preview/load for textual inspection later on.
    #[serde(default = "default_max_preview_bytes")]
    pub max_preview_bytes: u64,

    /// Directory names to exclude completely.
    #[serde(default)]
    pub exclude_dirs: Vec<String>,

    #[serde(default)]
    pub file_kinds: HashMap<FileKind, Vec<String>>,

}

impl ScanConfig {

    pub fn from_yaml_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_ref = path.as_ref();

        let text = fs::read_to_string(path_ref).with_context(|| {
            format!(
                "failed to read scan config from {}",
                path_ref.display()
            )
        })?;

        let config: Self = serde_yaml::from_str(&text).with_context(|| {
            format!(
                "failed to parse yaml config from {}",
                path_ref.display()
            )
        })?;

        Ok(config)
    }

    pub fn default_profile() -> Result<Self> {
        Self::from_yaml_str(DEFAULT_PROFILE_YAML)
    }

    pub fn single_cell_profile() -> Result<Self> {
        Self::from_yaml_str(SINGLE_CELL_PROFILE_YAML)
    }

    pub fn from_yaml_str(text: &str) -> Result<Self> {
        let config: Self = serde_yaml::from_str(text)
            .context("failed to parse embedded yaml scan config")?;

        Ok(config)
    }

    pub fn from_optional_yaml_file<P: AsRef<Path>>(
        path: Option<P>,
    ) -> Result<Self> {
        match path {
            Some(path) => Self::from_yaml_file(path),
            None => Self::default_profile(),
        }
    }

    pub fn write_default_profile<P: AsRef<Path>>(path: P) -> Result<()> {
        write_profile(path, DEFAULT_PROFILE_YAML)
    }

    pub fn write_single_cell_profile<P: AsRef<Path>>(path: P) -> Result<()> {
        write_profile(path, SINGLE_CELL_PROFILE_YAML)
    }

    pub fn classify_file_name(&self, file_name: &str) -> Option<FileKind> {
        let file_name_lower = file_name.to_ascii_lowercase();

        let mut best: Option<(usize, FileKind)> = None;

        for (kind, patterns) in &self.file_kinds {
            for pattern in patterns {
                let pattern_lower = pattern.to_ascii_lowercase();

                let matches = if pattern_lower.contains('.') {
                    file_name_lower.ends_with(&format!(".{pattern_lower}"))
                } else {
                    file_name_lower
                        .rsplit_once('.')
                        .map(|(_, ext)| ext == pattern_lower)
                        .unwrap_or(false)
                };

                if matches {
                    let score = pattern_lower.len();

                    if best
                        .as_ref()
                        .map(|(best_score, _)| score > *best_score)
                        .unwrap_or(true)
                    {
                        best = Some((score, kind.clone()));
                    }
                }
            }
        }

        best.map(|(_, kind)| kind)
    }

    pub fn dir_is_excluded(&self, dir_name: &str) -> bool {
        self.exclude_dirs
            .iter()
            .any(|candidate| candidate == dir_name)
    }
}

impl Default for ScanConfig {
    fn default() -> Self {
       Self::default_profile()
            .expect("embedded default profile should parse")
    }
}

fn default_max_preview_bytes() -> u64 {
    5_000_000
}

fn write_profile<P: AsRef<Path>>(path: P, yaml: &str) -> Result<()> {
    let path_ref = path.as_ref();

    if let Some(parent) = path_ref.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!("failed to create profile directory {}", parent.display())
        })?;
    }

    fs::write(path_ref, yaml).with_context(|| {
        format!("failed to write profile to {}", path_ref.display())
    })?;

    Ok(())
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_expected_values() {
        let config = ScanConfig::default();

        assert_eq!(config.name, "default");

        assert!(!config.include_hidden);
        assert!(!config.follow_symlinks);

        assert!(config.dir_is_excluded(".git"));
        assert!(config.dir_is_excluded("target"));

        assert!(config.extension_is_text("rs"));
        assert!(config.extension_is_text("ipynb"));

        assert!(config.extension_is_large_data("h5ad"));
        assert!(config.extension_is_large_data("bam"));

        assert!(!config.extension_is_text("bam"));
        assert!(!config.extension_is_large_data("rs"));
    }

    #[test]
    fn extension_matching_is_case_insensitive() {
        let config = ScanConfig::default();

        assert!(config.extension_is_text("RS"));
        assert!(config.extension_is_text("IpYnB"));

        assert!(config.extension_is_large_data("H5AD"));
        assert!(config.extension_is_large_data("rdata"));
    }

    #[test]
    fn excluded_directory_matching_works() {
        let config = ScanConfig::default();

        assert!(config.dir_is_excluded(".git"));
        assert!(config.dir_is_excluded("work"));

        assert!(!config.dir_is_excluded("src"));
        assert!(!config.dir_is_excluded("analysis"));
    }

    #[test]
    fn yaml_deserialization_works() {
        let yaml = r#"
name: test_profile

include_hidden: true
follow_symlinks: true
max_preview_bytes: 1234

exclude_dirs:
  - cache

large_data_extensions:
  - h5ad

text_extensions:
  - rs
  - py
"#;

        let config: ScanConfig =
            serde_yaml::from_str(yaml).expect("yaml should parse");

        assert_eq!(config.name, "test_profile");

        assert!(config.include_hidden);
        assert!(config.follow_symlinks);

        assert_eq!(config.max_preview_bytes, 1234);

        assert!(config.dir_is_excluded("cache"));

        assert!(config.extension_is_large_data("h5ad"));

        assert!(config.extension_is_text("rs"));
        assert!(config.extension_is_text("py"));
    }
}
