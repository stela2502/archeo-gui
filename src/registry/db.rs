use std::path::Path;

use anyhow::{Context, Result};
use chrono::Utc;
use rusqlite::{params, Connection};

use crate::ai::cluster_response::ClusterResponse;
use crate::registry::models::{
    AiAssumption, ArtifactGroup, ArtifactGroupKind, ArtifactGroupMember,
    PromptSnapshot, ScanRun, ValidationStatus,
};
use crate::registry::schema::SCHEMA_SQL;
use crate::registry::models::{BucketClassification, FileBucket};
use crate::scanner::config::ScanConfig;
use crate::scanner::file_entry::{FileEntry, FileKind};

pub struct RegistryDb {
    conn: Connection,
}

impl RegistryDb {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_ref = path.as_ref();

        if let Some(parent) = path_ref.parent() {
            std::fs::create_dir_all(parent).with_context(|| {
                format!("failed to create registry directory {}", parent.display())
            })?;
        }

        let conn = Connection::open(path_ref).with_context(|| {
            format!("failed to open sqlite database {}", path_ref.display())
        })?;

        conn.execute_batch(SCHEMA_SQL)
            .context("failed to initialize registry schema")?;

        Ok(Self { conn })
    }

    pub fn insert_scan_run(
        &self,
        root: &Path,
        config: &ScanConfig,
    ) -> Result<ScanRun> {
        let scan_run = ScanRun {
            id: uuid::Uuid::new_v4().to_string(),
            root_path: root.to_string_lossy().to_string(),
            profile_name: config.name.clone(),
            created_at: Utc::now().to_rfc3339(),
        };

        self.conn.execute(
            "INSERT INTO scan_runs
             (id, root_path, profile_name, created_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                scan_run.id,
                scan_run.root_path,
                scan_run.profile_name,
                scan_run.created_at
            ],
        )?;

        Ok(scan_run)
    }

    pub fn insert_file_entries(
        &mut self,
        scan_id: &str,
        mut entries: Vec<FileEntry>,
    ) -> Result<()> {
        let tx = self.conn.transaction()?;

        {
            let mut stmt = tx.prepare(
                "INSERT INTO file_entries
                 (id, scan_id, rel_path, file_name, extension, file_kind,
                  size_bytes, modified_at, included, exclusion_reason)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            )?;

            for entry in entries.iter_mut() {
                entry.scan_id = scan_id.to_string();

                stmt.execute(params![
                    entry.id,
                    entry.scan_id,
                    entry.rel_path.to_string_lossy().to_string(),
                    entry.file_name,
                    entry.extension,
                    entry.file_kind.to_string(),
                    entry.size_bytes as i64,
                    entry.modified_at,
                    entry.included as i64,
                    entry.exclusion_reason,
                ])?;
            }
        }

        tx.commit()?;

        Ok(())
    }

    pub fn naive_file_buckets_for_scan(
        &self,
        scan_id: &str,
        max_examples_per_bucket: usize,
    ) -> Result<Vec<FileBucket>> {
        let mut stmt = self.conn.prepare(
            "SELECT
                 file_kind,
                 extension,
                 COUNT(*) AS count,
                 COALESCE(SUM(size_bytes), 0) AS total_size_bytes
             FROM file_entries
             WHERE scan_id = ?1
               AND included = 1
               AND file_kind != 'directory'
             GROUP BY file_kind, extension
             ORDER BY count DESC, file_kind, extension",
        )?;

        let rows = stmt.query_map(params![scan_id], |row| {
            Ok(FileBucket {
                file_kind: row.get(0)?,
                extension: row.get(1)?,
                count: row.get::<_, i64>(2)? as usize,
                total_size_bytes: row.get::<_, i64>(3)? as u64,
                examples: Vec::new(),
                classification: BucketClassification::Unreviewed,
                user_note: String::new(),
                ai_note: None,
            })
        })?;

        let mut buckets = Vec::new();

        for row in rows {
            let mut bucket = row?;
            bucket.examples = self.examples_for_bucket(
                scan_id,
                &bucket.file_kind,
                bucket.extension.as_deref(),
                max_examples_per_bucket,
            )?;
            buckets.push(bucket);
        }

        Ok(buckets)
    }

    fn examples_for_bucket(
        &self,
        scan_id: &str,
        file_kind: &str,
        extension: Option<&str>,
        limit: usize,
    ) -> Result<Vec<String>> {
        if let Some(extension) = extension {
            let mut stmt = self.conn.prepare(
                "SELECT rel_path
                 FROM file_entries
                 WHERE scan_id = ?1
                   AND included = 1
                   AND file_kind = ?2
                   AND extension = ?3
                   AND file_kind != 'directory'
                 ORDER BY rel_path
                 LIMIT ?4",
            )?;

            let rows = stmt.query_map(
                params![scan_id, file_kind, extension, limit as i64],
                |row| row.get::<_, String>(0),
            )?;

            let mut examples = Vec::new();

            for row in rows {
                examples.push(row?);
            }

            Ok(examples)
        } else {
            let mut stmt = self.conn.prepare(
                "SELECT rel_path
                 FROM file_entries
                 WHERE scan_id = ?1
                   AND included = 1
                   AND file_kind = ?2
                   AND extension IS NULL
                   AND file_kind != 'directory'
                 ORDER BY rel_path
                 LIMIT ?3",
            )?;

            let rows = stmt.query_map(
                params![scan_id, file_kind, limit as i64],
                |row| row.get::<_, String>(0),
            )?;

            let mut examples = Vec::new();

            for row in rows {
                examples.push(row?);
            }

            Ok(examples)
        }
    }
    pub fn latest_scan_run(&self) -> Result<ScanRun> {
        self.conn
            .query_row(
                "SELECT id, root_path, profile_name, created_at
                 FROM scan_runs
                 ORDER BY created_at DESC
                 LIMIT 1",
                [],
                |row| {
                    Ok(ScanRun {
                        id: row.get(0)?,
                        root_path: row.get(1)?,
                        profile_name: row.get(2)?,
                        created_at: row.get(3)?,
                    })
                },
            )
            .context("no scan run found in registry")
    }

    pub fn file_entries_for_scan(
        &self,
        scan_id: &str,
    ) -> Result<Vec<FileEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, scan_id, rel_path, file_name, extension, file_kind,
                    size_bytes, modified_at, included, exclusion_reason
             FROM file_entries
             WHERE scan_id = ?1
             ORDER BY rel_path",
        )?;

        let rows = stmt.query_map(params![scan_id], |row| {
            Ok(FileEntry {
                id: row.get(0)?,
                scan_id: row.get(1)?,
                rel_path: std::path::PathBuf::from(row.get::<_, String>(2)?),
                file_name: row.get(3)?,
                extension: row.get(4)?,
                file_kind: row.get::<_, String>(5)?
                            .parse()
                            .unwrap_or(FileKind::Unknown),
                size_bytes: row.get::<_, i64>(6)? as u64,
                modified_at: row.get(7)?,
                included: row.get::<_, i64>(8)? != 0,
                exclusion_reason: row.get(9)?,
            })
        })?;

        let mut entries = Vec::new();

        for row in rows {
            entries.push(row?);
        }

        Ok(entries)
    }

    pub fn insert_prompt_snapshot(
        &self,
        scan_id: &str,
        model: &str,
        prompt: &str,
    ) -> Result<String> {
        let id = uuid::Uuid::new_v4().to_string();

        self.conn.execute(
            "INSERT INTO prompt_snapshots
             (id, scan_id, model, prompt, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                id,
                scan_id,
                model,
                prompt,
                Utc::now().to_rfc3339()
            ],
        )?;

        Ok(id)
    }

    pub fn insert_cluster_response(
        &mut self,
        scan_id: &str,
        model: &str,
        prompt_snapshot_id: &str,
        response: ClusterResponse,
    ) -> Result<()> {
        let tx = self.conn.transaction()?;

        for group in response.groups {
            let group_id = uuid::Uuid::new_v4().to_string();

            tx.execute(
                "INSERT INTO artifact_groups
                 (id, scan_id, name, group_kind, ai_statement,
                  confidence, human_status)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    group_id,
                    scan_id,
                    group.name,
                    group.group_kind.to_string(),
                    group.ai_statement,
                    group.confidence,
                    ValidationStatus::Unreviewed.to_string(),
                ],
            )?;

            for member in group.members {
                let file_id: String = tx.query_row(
                    "SELECT id FROM file_entries
                     WHERE scan_id = ?1 AND rel_path = ?2
                     LIMIT 1",
                    params![scan_id, member.rel_path],
                    |row| row.get(0),
                )?;

                tx.execute(
                    "INSERT OR IGNORE INTO artifact_group_members
                     (group_id, file_id, role)
                     VALUES (?1, ?2, ?3)",
                    params![group_id, file_id, member.role],
                )?;
            }

            for statement in group.assumptions {
                let assumption_id = uuid::Uuid::new_v4().to_string();

                tx.execute(
                    "INSERT INTO ai_assumptions
                     (id, scan_id, scope_kind, scope_id, statement, confidence,
                      model, prompt_snapshot_id, human_status)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                    params![
                        assumption_id,
                        scan_id,
                        "artifact_group",
                        group_id,
                        statement,
                        group.confidence,
                        model,
                        prompt_snapshot_id,
                        ValidationStatus::Unreviewed.to_string(),
                    ],
                )?;
            }
        }

        tx.commit()?;

        Ok(())
    }
}
