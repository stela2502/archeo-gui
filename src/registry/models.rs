use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScanRun {
    pub id: String,
    pub root_path: String,
    pub profile_name: String,
    pub created_at: String,
}


/// FileBucket
/// = local mechanical summary
/// = "notebook / ipynb / 82 files"
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileBucket {
    pub file_kind: String,
    pub extension: Option<String>,
    pub count: usize,
    pub total_size_bytes: u64,
    pub examples: Vec<String>,
}

impl FileBucket {
    pub fn display_name(&self) -> String {
        match &self.extension {
            Some(ext) => {
                format!(
                    "{} / {}",
                    self.file_kind,
                    ext
                )
            }

            None => self.file_kind.clone(),
        }
    }

    pub fn human_size_mb(&self) -> f64 {
        self.total_size_bytes as f64
            / 1024.0
            / 1024.0
    }
    pub fn len(&self) -> usize {
        self.count
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
}

impl std::fmt::Display for FileBucket {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        writeln!(
            f,
            "{} / {}",
            self.file_kind,
            self.extension
                .as_deref()
                .unwrap_or("no_extension")
        )?;

        writeln!(f, "  count: {}", self.count)?;

        writeln!(
            f,
            "  total_size_mb: {:.2}",
            self.total_size_bytes as f64
                / 1024.0
                / 1024.0
        )?;

        if !self.examples.is_empty() {
            writeln!(f, "  examples:")?;

            for example in self.examples.iter().take(3) {
                writeln!(f, "    - {}", example)?;
            }
        }

        Ok(())
    }
}

/// ArtifactGroup
/// = semantic AI/human interpretation
/// = "Analysis notebooks involved in RF clustering"
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ArtifactGroup {
    pub id: String,
    pub scan_id: String,
    pub name: String,
    pub group_kind: ArtifactGroupKind,
    pub ai_statement: String,
    pub confidence: f32,
    pub human_status: ValidationStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArtifactGroupMember {
    pub group_id: String,
    pub file_id: String,
    pub role: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AiAssumption {
    pub id: String,
    pub scan_id: String,
    pub scope_kind: ScopeKind,
    pub scope_id: String,
    pub statement: String,
    pub confidence: f32,
    pub model: String,
    pub prompt_snapshot_id: String,
    pub human_status: ValidationStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PromptSnapshot {
    pub id: String,
    pub scan_id: String,
    pub model: String,
    pub prompt: String,
    pub created_at: String,
}

#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Display, EnumString,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ArtifactGroupKind {
    RawData,
    AnalysisScripts,
    Notebooks,
    ProcessedData,
    ResultFiles,
    Figures,
    Logs,
    Archives,
    Temporary,
    Publication,
    Mixed,
    Unknown,
}

#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Display, EnumString,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ScopeKind {
    ScanRun,
    ArtifactGroup,
    FileEntry,
    PromptSnapshot,
    Unknown,
}

#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Display, EnumString,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ValidationStatus {
    Unreviewed,
    Accepted,
    Rejected,
    NeedsReview,
}
