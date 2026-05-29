use std::path::Path;

use anyhow::{Context, Result};
use chrono::Utc;
use rusqlite::{params, Connection};

use crate::ai::cluster_response::ClusterResponse;
use crate::registry::models::{
    AiAssumption, ArtifactGroup, ArtifactGroupKind, ArtifactGroupMember,
    BucketClassification, FileBucket, PromptSnapshot, ScanRun, 
    ValidationStatus,
};
use crate::registry::schema::SCHEMA_SQL;
use crate::scanner::config::ScanConfig;
use crate::scanner::file_entry::{FileEntry, FileKind};

#[derive(Debug)]
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

    pub fn load_bucket_classifications(
        &self,
        scan_id: &str,
        buckets: &mut [FileBucket],
    ) -> Result<()> {
        let mut stmt = self.conn.prepare(
            "SELECT classification, user_note, ai_note
             FROM bucket_classifications
             WHERE scan_id = ?1
               AND file_kind = ?2
               AND extension_key = ?3",
        )?;

        for bucket in buckets.iter_mut() {
            let key = extension_key(bucket.extension.as_deref());

            let result = stmt.query_row(
                params![
                    scan_id,
                    bucket.file_kind.as_str(),
                    key,
                ],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, Option<String>>(2)?,
                    ))
                },
            );

            match result {
                Ok((classification, user_note, ai_note)) => {
                    bucket.classification = classification
                        .parse()
                        .unwrap_or(BucketClassification::NeedsInspection);

                    bucket.user_note = user_note;
                    bucket.ai_note = ai_note;
                }

                Err(rusqlite::Error::QueryReturnedNoRows) => {
                    bucket.classification = BucketClassification::NeedsInspection;
                }

                Err(err) => return Err(err.into()),
            }
        }

        Ok(())
    } 

    pub fn save_bucket_classification(
        &self,
        scan_id: &str,
        bucket: &FileBucket,
    ) -> Result<()> {
        let key = extension_key(bucket.extension.as_deref());

        self.conn.execute(
            "INSERT INTO bucket_classifications
             (scan_id, file_kind, extension_key, classification,
              user_note, ai_note, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(scan_id, file_kind, extension_key)
             DO UPDATE SET
                classification = excluded.classification,
                user_note = excluded.user_note,
                ai_note = excluded.ai_note,
                updated_at = excluded.updated_at",
            params![
                scan_id,
                bucket.file_kind,
                key,
                bucket.classification.to_string(),
                bucket.user_note,
                bucket.ai_note,
                Utc::now().to_rfc3339(),
            ],
        )?;

        Ok(())
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

    pub fn file_entries_for_bucket(
        &self,
        scan_id: &str,
        file_kind: &str,
        extension: Option<&str>,
    ) -> Result<Vec<FileEntry>> {
        let mut entries = Vec::new();

        if let Some(extension) = extension {
            let mut stmt = self.conn.prepare(
                "SELECT id, scan_id, rel_path, file_name, extension, file_kind,
                        size_bytes, modified_at, included, exclusion_reason
                 FROM file_entries
                 WHERE scan_id = ?1
                   AND included = 1
                   AND file_kind = ?2
                   AND extension = ?3
                 ORDER BY rel_path",
            )?;

            let rows = stmt.query_map(
                params![scan_id, file_kind, extension],
                row_to_file_entry,
            )?;

            for row in rows {
                entries.push(row?);
            }
        } else {
            let mut stmt = self.conn.prepare(
                "SELECT id, scan_id, rel_path, file_name, extension, file_kind,
                        size_bytes, modified_at, included, exclusion_reason
                 FROM file_entries
                 WHERE scan_id = ?1
                   AND included = 1
                   AND file_kind = ?2
                   AND extension IS NULL
                 ORDER BY rel_path",
            )?;

            let rows = stmt.query_map(
                params![scan_id, file_kind],
                row_to_file_entry,
            )?;

            for row in rows {
                entries.push(row?);
            }
        }

        Ok(entries)
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

    /// returns an Err if the Databse interaction had an error
    /// or None if there was no entry in the database
    pub fn latest_scan_run(
        &self,
    ) -> Result<Option<ScanRun>> {
        let result = self.conn.query_row(
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
        );

        match result {
            Ok(scan_run) => Ok(Some(scan_run)),

            Err(rusqlite::Error::QueryReturnedNoRows) => {
                Ok(None)
            }

            Err(err) => Err(err.into()),
        }
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

    pub fn scan_run_by_id(
        &self,
        scan_id: &str,
    ) -> Result<Option<ScanRun>> {
        let result = self.conn.query_row(
            "SELECT id, root_path, profile_name, created_at
             FROM scan_runs
             WHERE id = ?1",
            params![scan_id],
            |row| {
                Ok(ScanRun {
                    id: row.get(0)?,
                    root_path: row.get(1)?,
                    profile_name: row.get(2)?,
                    created_at: row.get(3)?,
                })
            },
        );

        match result {
            Ok(scan_run) => Ok(Some(scan_run)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(err) => Err(err.into()),
        }
    }
}


fn extension_key(extension: Option<&str>) -> String {
    extension
        .unwrap_or("no_extension")
        .to_string()
}


fn row_to_file_entry(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<FileEntry> {
    Ok(FileEntry {
        id: row.get(0)?,
        scan_id: row.get(1)?,
        rel_path: std::path::PathBuf::from(row.get::<_, String>(2)?),
        file_name: row.get(3)?,
        extension: row.get(4)?,
        file_kind: row
            .get::<_, String>(5)?
            .parse()
            .unwrap_or(FileKind::Unknown),
        size_bytes: row.get::<_, i64>(6)? as u64,
        modified_at: row.get(7)?,
        included: row.get::<_, i64>(8)? != 0,
        exclusion_reason: row.get(9)?,
    })
}