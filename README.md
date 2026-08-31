# archeo-gui

**Local-first, AI-assisted archaeology for scientific research folders.**

Research projects rarely stay tidy.

A directory that started with a FASTQ file and an analysis script eventually contains notebooks, intermediate matrices, figures, old parameter sweeps, logs, renamed results, publication exports, mysterious archives, and that one `final_results_really_final_v3` directory nobody wants to delete.

`archeo-gui` is a Rust application for making sense of that history.

It scans complex research directories, builds a persistent **SQLite registry** of their artifacts, classifies files using configurable scientific profiles, and can use a **local LLM through Ollama** to infer higher-level groups such as raw data, analysis code, processed datasets, figures, logs, publication material, and abandoned experiments.

The goal is not to reorganize your filesystem automatically.

The goal is to **understand what is already there**.

> **Status:** Experimental / under active development.
> The core scanner, registry, CLI, GUI, and first-pass local-AI clustering infrastructure are implemented, but interfaces and workflows may still change.

---

## Why archeo-gui?

Scientific directories encode a surprising amount of project history.

File names, extensions, directory structure, modification times, repeated naming patterns, output formats, and neighboring artifacts often reveal:

* which files are raw inputs,
* which scripts belong to an analysis,
* which outputs were generated together,
* which experiments are probably obsolete,
* where figures came from,
* which files may belong to a manuscript,
* and which parts of the directory nobody remembers anymore.

Traditional file browsers show files.

`archeo-gui` tries to reconstruct the **semantic structure of the research project** around them.

It combines deterministic filesystem inspection with optional local AI reasoning, while keeping the underlying evidence available for human inspection.

---

## Design principles

### Local first

Your research folder stays on your machine.

Scanning and indexing are performed locally, and the project registry is stored as SQLite inside the project by default:

`<project>/.archeo/archeo.sqlite`

AI-assisted clustering currently uses a locally running **Ollama** instance.

The first-pass clustering step operates on compact filesystem metadata rather than uploading file contents to an external service.

### Evidence before AI

`archeo-gui` does not begin by throwing an entire research directory at a language model.

Instead, it first builds a deterministic representation of the filesystem:

**filesystem → classification → registry → buckets → semantic inference**

This keeps expensive or probabilistic reasoning separated from the underlying evidence.

The AI proposes an interpretation; it does not replace the registry.

### Persistent provenance

Scans are stored rather than existing only as transient GUI state.

This makes the SQLite registry a foundation for future provenance-aware workflows, comparisons between scans, artifact relationships, and reproducible project archaeology.

### Scientific files are first-class citizens

The bundled profiles recognize common research artifacts including:

* R, Python, Rust, shell and Nextflow source code
* Jupyter notebooks
* YAML, TOML, JSON and workflow configuration
* CSV/TSV/MTX tables
* FASTQ, BAM, CRAM and SAM files
* HDF5, RDS, RData and Parquet data
* h5ad, loom, h5mu and Zarr single-cell objects
* 10x-style matrices and barcode/feature tables
* figures and PDFs
* logs
* archives
* manuscript-oriented text formats

The classification system is YAML-configurable, so project-specific conventions do not need to be hard-coded into the application.

---

## What it does

At the moment, `archeo-gui` provides four main building blocks.

### 1. Filesystem scanning

The scanner recursively walks a research directory and records its artifacts.

Configurable scan profiles control:

* excluded directories,
* hidden files,
* symbolic links,
* preview limits,
* file-type classification,
* and domain-specific conventions.

Large generated directories such as `.git`, `target`, temporary workflow folders, and other known noise can therefore be excluded before they dominate the project model.

### 2. Persistent SQLite registry

Each scan is registered in SQLite.

Rather than repeatedly rediscovering the same filesystem from scratch, downstream functionality can work against a persistent representation of the project.

This also provides the basis for tracking how a research directory evolves over time.

### 3. Artifact bucketing

Files are summarized into compact buckets based on their observed properties.

These buckets provide a bridge between raw filesystem structure and semantic interpretation: hundreds or thousands of individual files can first be reduced to a much smaller representation of recurring artifact types.

### 4. Local AI-assisted clustering

An optional second pass sends those compact bucket descriptions to an Ollama model.

The model is asked to propose semantic groups such as:

* raw data
* analysis scripts
* notebooks
* processed data
* result files
* figures
* logs
* archives
* temporary artifacts
* publication material

The response, assumptions, confidence values, and prompt snapshot are stored in the registry.

This makes the AI layer an **auditable interpretation of the project**, rather than an invisible source of truth.

---

## GUI

`archeo-gui` includes a native desktop interface built with `egui`/`eframe`.

The GUI is intended to turn the registry into an interactive research-project explorer.

Current components include:

* project loading
* bucket navigation
* classification-aware views
* sorting and filtering
* file previews
* text and Markdown viewing
* search
* scan status
* artifact inspection
* background worker infrastructure

The GUI and CLI operate on the same underlying scanner and registry model.

---

## Command-line interface

Two binaries are provided:

* `archeo` — command-line scanner and clustering interface
* `archeo-gui` — native graphical interface

### Scan a project

```
archeo scan \
    --root ~/projects/my_analysis \
    --profile profiles/single_cell.yaml
```

Unless another database is specified, this creates:

```
~/projects/my_analysis/.archeo/archeo.sqlite
```

If no profile is supplied, the embedded default profile is used.

### Cluster the project with a local LLM

First make sure Ollama is running and the desired model is available.

Then:

```
archeo cluster \
    --root ~/projects/my_analysis \
    --model llama3.1
```

Alternatively, address the registry directly:

```
archeo cluster \
    --db ~/projects/my_analysis/.archeo/archeo.sqlite \
    --model llama3.1
```

The Ollama API defaults to:

```
http://127.0.0.1:11434/api
```

A different endpoint and timeout can be supplied through the CLI.

### Create an editable scan profile

The built-in profiles can be exported and customized:

```
archeo init-profile \
    --profile default \
    --out my_profile.yaml
```

For single-cell projects:

```
archeo init-profile \
    --profile single-cell \
    --out my_single_cell_profile.yaml
```

---

## Single-cell research folders

One of the original use cases for `archeo-gui` is archaeology of bioinformatics projects, particularly single-cell sequencing analyses.

These projects tend to accumulate several generations of:

* raw sequencing data,
* 10x matrices,
* Seurat and AnnData objects,
* preprocessing outputs,
* notebooks,
* R scripts,
* Python scripts,
* workflow definitions,
* cluster annotations,
* differential-expression tables,
* QC plots,
* publication figures,
* and intermediate experiments.

The included `single_cell` profile understands many of these conventions and excludes several common high-volume generated directories.

For example:

```
archeo scan \
    --root ~/old_scRNA_project \
    --profile profiles/single_cell.yaml
```

The profile is deliberately editable. A lab with established naming conventions can therefore turn those conventions into explicit classification rules rather than relying entirely on heuristics or AI.

---

## Architecture

At a high level:

```
                  Research folder
                        │
                        ▼
                ┌───────────────┐
                │    Scanner    │
                └───────┬───────┘
                        │
              YAML classification
                        │
                        ▼
                ┌───────────────┐
                │ SQLite registry│
                └───────┬───────┘
                        │
                  file buckets
                        │
              ┌─────────┴─────────┐
              │                   │
              ▼                   ▼
        Native GUI          Local LLM / Ollama
                                  │
                                  ▼
                         semantic artifact groups
                                  │
                                  ▼
                           SQLite registry
```

The code is correspondingly split into a few major subsystems:

```
src/
├── ai/          local LLM prompting and response handling
├── gui/         egui/eframe desktop interface
├── registry/    persistent SQLite project model
├── scanner/     filesystem scanning and classification
├── bin/         CLI and GUI binaries
├── cli.rs
└── lib.rs
```

The separation is intentional: filesystem discovery, persistent state, visualization, and probabilistic interpretation should remain independently testable components.

---

## Building

`archeo-gui` is written in Rust using the 2024 edition.

Clone the repository and build it with Cargo:

```
git clone https://github.com/stela2502/archeo-gui
cd archeo-gui
cargo build --release
```

The resulting binaries are available under:

```
target/release/archeo
target/release/archeo-gui
```

For development:

```
cargo run --bin archeo -- --help
```

or:

```
cargo run --bin archeo-gui
```

---

## AI is optional

The filesystem scanner and SQLite registry do **not** require an LLM.

This is intentional.

A project should remain inspectable even when:

* Ollama is not installed,
* no suitable model is available,
* AI inference is undesirable,
* or the user simply wants deterministic filesystem exploration.

AI-assisted clustering is an additional semantic layer on top of the registry.

It is not the foundation of the application.

---

## Privacy

Research directories can contain unpublished results and sensitive project information.

For that reason, the current AI workflow is designed around **local inference through Ollama**.

The initial clustering prompt contains summarized file metadata rather than arbitrary file contents.

Nevertheless, this project is experimental software. Users working with sensitive or regulated data should inspect the code and configuration and verify that the workflow meets their local data-governance requirements before use.

---

## Current status

`archeo-gui` is currently an experimental project and under active development.

Implemented foundations include:

* recursive project scanning
* configurable YAML profiles
* scientific file classification
* SQLite-backed scan registry
* file bucketing
* native GUI
* file and text inspection
* search infrastructure
* local Ollama integration
* structured AI clustering responses
* persistent AI prompt/response records

There is considerably more that project archaeology can become.

Possible future directions include richer artifact relationships, scan-to-scan comparison, provenance reconstruction, workflow detection, duplicate/obsolete artifact discovery, interactive correction of inferred groups, and deeper content-aware analysis where explicitly requested by the user.

The long-term idea is simple:

> **Turn an old research directory from a pile of files into an explorable model of how the science was done.**

---

## What archeo-gui is not

`archeo-gui` is **not** a workflow manager.

It is not trying to replace Nextflow, Snakemake, Git, an electronic lab notebook, or a proper data-management strategy.

Quite the opposite.

It is intended for the situation where some or all of those things were missing, incomplete, changed over time, or arrived only after the project had already accumulated several years of history.

In other words:

**workflow management is for the project you are about to start;
project archaeology is for the one you just inherited.**

---

## Contributing

The project is still evolving rapidly, so bug reports, real-world research-folder examples, classification ideas, and contributions are welcome.

In particular, profiles for additional scientific domains could make the scanner useful well beyond its initial bioinformatics and single-cell use cases.

---

License

archeo-gui is available under a dual-use licensing model:

    Teaching, academic research, and non-commercial use: free of charge.

    Commercial or business use: requires a commercial license.

This means universities, educators, students, and academic researchers can use archeo-gui freely for teaching and non-commercial research, while companies and other commercial users must obtain a paid license.

See LICENSE and COMMERCIAL.md for the exact licensing terms and commercial licensing information.

---

## Author

**Stefan Lang**

Built for scientists who have ever opened an old project directory and thought:


