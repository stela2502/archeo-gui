use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum BuiltinProfile {
    Default,
    SingleCell,
}


#[derive(Parser, Debug)]
#[command(
    name = "archeo-gui",
    version,
    about = "Scan messy research folders, register their artifacts, and cluster them with local AI.",
    long_about = "\
archeo-gui builds a persistent SQLite registry for a research folder.

The first workflow is:

  archeo-gui scan --root <FOLDER> --profile <PROFILE.yaml>
  archeo-gui cluster --db <FOLDER>/.archeo/archeo.sqlite --model <OLLAMA_MODEL>

The scan command walks a folder, applies YAML filters, classifies files by type,
and stores a stable registry in .archeo/archeo.sqlite.

The cluster command reads that registry, sends only compact file metadata to a
local Ollama model, and stores proposed logical artifact groups such as raw data,
analysis notebooks, result files, logs, archives, and temporary experiments.

No file contents are sent during the first clustering step."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[command(
        about = "Scan a folder and register files in SQLite.",
        long_about = "\
Scan a research folder and create/update a local SQLite registry.

This command:
  - walks the folder recursively
  - applies a YAML filter profile
  - classifies files by extension and path
  - records included and excluded files
  - stores everything in <root>/.archeo/archeo.sqlite unless --db is given

Example:
  archeo-gui scan \\
    --root ~/project/my_analysis \\
    --profile profiles/single_cell.yaml"
    )]
    Scan {
        /// Root folder to scan.
        #[arg(long)]
        root: PathBuf,

        /// YAML scan/filter profile.
        ///
        /// If omitted, the embedded default profile is used.
        #[arg(long)]
        profile: Option<PathBuf>,

        /// Optional SQLite database path.
        ///
        /// Defaults to <root>/.archeo/archeo.sqlite.
        #[arg(long)]
        db: Option<PathBuf>,
    },

    #[command(
        about = "Use local AI to cluster registered files into artifact groups.",
        long_about = "\
Cluster the files registered by a previous scan.

This command:
  - opens the SQLite registry
  - loads the latest scan run
  - builds a compact metadata-only prompt
  - asks a local Ollama model to propose logical artifact groups
  - stores groups, memberships, assumptions, and the prompt snapshot

Example:
  archeo-gui cluster \\
    --db ~/project/my_analysis/.archeo/archeo.sqlite \\
    --model llama3.1"
    )]
    Cluster {
        /// Root project folder.
        ///
        /// If supplied, the database is assumed to be:
        /// <root>/.archeo/archeo.sqlite
        #[arg(long, conflicts_with = "db")]
        root: Option<PathBuf>,
    
        /// Explicit SQLite database path.
        #[arg(long, conflicts_with = "root")]
        db: Option<PathBuf>,

        /// Ollama model name.
        #[arg(long, default_value = "llama3.1")]
        model: String,

        /// Ollama API base URL.
        ///
        /// This should be the API base, not /generate.
        #[arg(long, default_value = "http://127.0.0.1:11434/api")]
        ollama_url: String,

        /// Bigger folders need longer time to process:
        #[arg(long, default_value_t = 1800)]
        timeout_seconds: u64,
    },
    #[command(
        about = "Write an editable built-in YAML profile to disk.",
        long_about = "\
Write one of the built-in scan profiles to disk.

This is useful when you want to start from the default rules and edit them
for a specific research folder.

Examples:
  archeo-gui init-profile --out profiles/default.yaml
  archeo-gui init-profile --profile single-cell --out profiles/single_cell.yaml"
    )]
    InitProfile {
        /// Built-in profile to write.
        #[arg(long, default_value = "default")]
        profile: BuiltinProfile,

        /// Output YAML path.
        #[arg(long)]
        out: PathBuf,
    },

}
