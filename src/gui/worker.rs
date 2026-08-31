use std::path::PathBuf;
use std::thread;

use anyhow::Result;
use mapping_info::MappingInfo;

use crate::gui::state::{GuiState, WizardStep, SearchMode};
use crate::registry::db::RegistryDb;
use crate::registry::models::{FileBucket, ScanRun};
use crate::scanner::config::ScanConfig;
use crate::scanner::scan::scan_folder;
use crate::gui::search::text_file::{SearchOptions, SearchHit, search_file_safely};
use crate::gui::state::{WorkspaceViewer, MarkdownDraft};



#[derive(Debug)]
pub enum GuiWorkerMessage {
    Started(String),

    ScanFinished {
        scan_run: ScanRun,
        buckets: Vec<FileBucket>,
        status: String,
    },

    SearchFinished {
        query: String,
        hits: Vec<SearchHit>,
        status: String,
    },

    AiFinished {
        answer: String,
        status: String,
    },

    Failed(String),
}

#[derive(Debug)]
pub enum GuiWorkerJob {
    Rescan {
        root: PathBuf,
        db_path: PathBuf,
    },

    Recluster {
        db_path: PathBuf,
        scan_id: String,
    },

    FindInBucket {
	    db_path: PathBuf,
	    scan_id: String,
	    file_kind: String,
	    extension: Option<String>,
	    needle: String,
	    use_regex: bool,
	    case_insensitive: bool,
	},

    AskAi {
        prompt: String,
        model: String,
    },

}

pub fn poll(state: &mut GuiState) {
    let Some(rx) = &state.worker_rx else {
        return;
    };

    let Ok(message) = rx.try_recv() else {
        return;
    };

    state.worker_rx = None;
    state.worker_busy = false;
    state.scan_in_progress = false;
    state.ai_busy = false;

    apply_message(state, message);
}

pub fn dispatch(state: &mut GuiState) {
    if state.worker_busy {
        return;
    }

    let Some(job) = state.pending_job.take() else {
        return;
    };

    let (tx, rx) = crossbeam_channel::unbounded();

    state.worker_rx = Some(rx);
    state.worker_busy = true;

    match &job {
        GuiWorkerJob::Rescan { .. } => {
            state.scan_in_progress = true;
            state.set_status("Scanning in background...");
        }

        GuiWorkerJob::AskAi { .. } => {
            state.ai_busy = true;
            state.set_status("AI request running...");
        }

        GuiWorkerJob::Recluster { .. } => {
            state.ai_busy = true;
            state.set_status("Clustering in background...");
        }

        GuiWorkerJob::FindInBucket { .. } => {
            state.set_status("Searching bucket in background...");
        }
    }

    thread::spawn(move || {
        let message = run_job(job);
        let _ = tx.send(message);
    });
}

fn apply_message(
    state: &mut GuiState,
    message: GuiWorkerMessage,
) {
    match message {
        GuiWorkerMessage::Started(status) => {
            state.set_status(status);
        }

        GuiWorkerMessage::ScanFinished {
            scan_run,
            buckets,
            status,
        } => {
            state.scan_run = Some(scan_run);
            //state.current_root = Some(folder.clone());
            state.buckets = buckets;
            state.selected_bucket = None;
            state.wizard_step = WizardStep::Ready;
            state.set_status(status);
        }

        GuiWorkerMessage::SearchFinished {
            hits,
            status,
            query,
        } => {

            state.tabs.push(WorkspaceViewer::Search {
                query,
                hits,
            });
            state.active_tab = Some(state.tabs.len() - 1);

            state.set_status(status);
        }

        GuiWorkerMessage::AiFinished {
            answer,
            status,
        } => {
            state.last_ai_answer = Some(answer);
            state.set_status(status);
        }

        GuiWorkerMessage::Failed(err) => {
            state.set_status(format!("Worker failed: {err}"));
        }
    }
}

fn run_job(job: GuiWorkerJob) -> GuiWorkerMessage {
    match job {
        GuiWorkerJob::Rescan { root, db_path } => {
            match scan_project(root, db_path) {
                Ok((scan_run, buckets, status)) => {
                    GuiWorkerMessage::ScanFinished {
                        scan_run,
                        buckets,
                        status,
                    }
                }

                Err(err) => {
                    GuiWorkerMessage::Failed(err.to_string())
                }
            }
        }

        GuiWorkerJob::Recluster {
            db_path: _,
            scan_id: _,
        } => {
            GuiWorkerMessage::Failed(
                "recluster worker is not implemented yet".to_string(),
            )
        }

        GuiWorkerJob::FindInBucket {
            db_path,
		    scan_id,
		    file_kind,
		    extension,
		    needle,
		    use_regex,
		    case_insensitive,
		} => {
            let query = format!("{}",needle );
            let fk = format!("{}",file_kind);
            let ext = format!("{:?}",extension );
            match run_find_in_bucket(
    		    db_path,
    		    scan_id,
    		    file_kind,
    		    extension,
    		    needle,
    		    use_regex,
    		    case_insensitive,
    		) {
    		    Ok((hits, status)) => {
                    eprintln!(
                        "search bucket kind={:?} ext={:?}",fk,ext
                    );
                    GuiWorkerMessage::SearchFinished {
                    query,
    		        hits,
    		        status,
    		      }
                },
    		    Err(err) => GuiWorkerMessage::Failed(err.to_string()),
    		}
        },

        GuiWorkerJob::AskAi {
            prompt: _,
            model: _,
        } => {
            GuiWorkerMessage::Failed(
                "AI worker is not implemented yet".to_string(),
            )
        }
    }
}

fn run_find_in_bucket(
    db_path: PathBuf,
    scan_id: String,
    file_kind: String,
    extension: Option<String>,
    needle: String,
    use_regex: bool,
    case_insensitive: bool,
) -> Result<(Vec<SearchHit>, String)> {
    let registry = RegistryDb::open(&db_path)?;

    let scan_run = registry
        .scan_run_by_id(&scan_id)?
        .ok_or_else(|| anyhow::anyhow!("scan run not found: {scan_id}"))?;

    let root = PathBuf::from(scan_run.root_path);

    let entries = registry.file_entries_for_bucket(
        &scan_id,
        &file_kind,
        extension.as_deref(),
    )?;

    let files_n = entries.len();

    let options = SearchOptions {
        needle,
        use_regex,
        case_insensitive,
        context_lines: 2,
        max_file_size_bytes: 10 * 1024 * 1024,
    };

    let mut out: Vec<SearchHit> = Vec::new();

    for entry in entries {
        let hits = search_file_safely(
            &root,
            &entry,
            &options,
        )?;

        out.extend(hits);
    }

    let status = format!("Search finished: {} hits in {} files", out.len(), files_n );

    Ok((out, status))

}

fn scan_project(
    root: PathBuf,
    db_path: PathBuf,
) -> Result<(ScanRun, Vec<FileBucket>, String)> {
    let config = ScanConfig::default_profile()?;
    let mut mapping_info = MappingInfo::new(None, 1.0, 1);

    let entries = scan_folder(&root, &config, &mut mapping_info)?;

    let mut registry = RegistryDb::open(&db_path)?;

    let scan_run = registry.insert_scan_run(&root, &config)?;

    registry.insert_file_entries(&scan_run.id, entries)?;

    let buckets =
        registry.naive_file_buckets_for_scan(&scan_run.id, 3)?;

    Ok((scan_run, buckets, format!("{mapping_info}")))
}