//! Text-file search with Sublime-style context.
//!
//! This module provides the core search logic:
//! - search a string
//! - search a file safely
//! - return structured hits with line numbers and surrounding context

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::scanner::file_entry::FileEntry;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchOptions {
    pub needle: String,
    pub use_regex: bool,
    pub case_insensitive: bool,
    pub context_lines: usize,
    pub max_file_size_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchHit {
    pub path: PathBuf,
    pub line_number: usize,
    pub matched_line: String,
    pub context_before: Vec<(usize, String)>,
    pub context_after: Vec<(usize, String)>,
}

impl SearchHit {
    pub fn to_display_string(&self) -> String {
        format!(
            "{}:{}: {}",
            self.path.display(),
            self.line_number,
            self.matched_line
        )
    }
}

pub fn search_file_safely(
    root: &Path,
    entry: &FileEntry,
    options: &SearchOptions,
) -> Result<Vec<SearchHit>> {
    if entry.size_bytes > options.max_file_size_bytes {
        return Ok(Vec::new());
    }

    let path = root.join(&entry.rel_path);

    let metadata = match fs::metadata(&path) {
        Ok(metadata) => metadata,
        Err(_) => return Ok(Vec::new()),
    };

    if !metadata.is_file() {
        return Ok(Vec::new());
    }

    if metadata.len() > options.max_file_size_bytes {
        return Ok(Vec::new());
    }

    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(_) => return Ok(Vec::new()),
    };

    search_text(&entry.rel_path, &text, options)
}

pub fn search_text_file(
    path: &Path,
    options: &SearchOptions,
) -> Result<Vec<SearchHit>> {
    let metadata = fs::metadata(path)
        .with_context(|| format!("failed to stat {}", path.display()))?;

    if metadata.len() > options.max_file_size_bytes {
        return Ok(Vec::new());
    }

    let text = fs::read_to_string(path)
        .with_context(|| format!("failed to read {}", path.display()))?;

    search_text(path, &text, options)
}

pub fn search_text(
    path: &Path,
    text: &str,
    options: &SearchOptions,
) -> Result<Vec<SearchHit>> {
    if options.needle.is_empty() {
        return Ok(Vec::new());
    }

    let lines: Vec<&str> = text.lines().collect();
    let matcher = Matcher::new(options)?;

    let mut hits = Vec::new();

    for (index, line) in lines.iter().enumerate() {
        if !matcher.is_match(line) {
            continue;
        }

        let start = index.saturating_sub(options.context_lines);
        let end = (index + options.context_lines + 1).min(lines.len());

        let context_before = (start..index)
            .map(|i| (i + 1, lines[i].to_string()))
            .collect();

        let context_after = ((index + 1)..end)
            .map(|i| (i + 1, lines[i].to_string()))
            .collect();

        hits.push(SearchHit {
            path: path.to_path_buf(),
            line_number: index + 1,
            matched_line: line.to_string(),
            context_before,
            context_after,
        });
    }

    Ok(hits)
}

enum Matcher {
    Plain {
        needle: String,
        case_insensitive: bool,
    },
    Regex(regex::Regex),
}

impl Matcher {
    fn new(options: &SearchOptions) -> Result<Self> {
        if options.use_regex {
            let mut builder = regex::RegexBuilder::new(&options.needle);
            builder.case_insensitive(options.case_insensitive);

            let regex = builder
                .build()
                .with_context(|| format!("invalid regex: {}", options.needle))?;

            Ok(Self::Regex(regex))
        } else {
            let needle = if options.case_insensitive {
                options.needle.to_lowercase()
            } else {
                options.needle.clone()
            };

            Ok(Self::Plain {
                needle,
                case_insensitive: options.case_insensitive,
            })
        }
    }

    fn is_match(&self, line: &str) -> bool {
        match self {
            Self::Plain {
                needle,
                case_insensitive,
            } => {
                if *case_insensitive {
                    line.to_lowercase().contains(needle)
                } else {
                    line.contains(needle)
                }
            }

            Self::Regex(regex) => regex.is_match(line),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opts(needle: &str) -> SearchOptions {
        SearchOptions {
            needle: needle.to_string(),
            use_regex: false,
            case_insensitive: false,
            context_lines: 2,
            max_file_size_bytes: 1024 * 1024,
        }
    }

    #[test]
    fn finds_plain_text_with_context() {
        let text = "\
line 1
line 2
ggsave(\"ABL1.png\")
line 4
line 5
line 6";

        let hits = search_text(
            Path::new("script.R"),
            text,
            &opts("ggsave"),
        )
        .unwrap();

        assert_eq!(hits.len(), 1);

        let hit = &hits[0];

        assert_eq!(hit.line_number, 3);
        assert_eq!(hit.matched_line, "ggsave(\"ABL1.png\")");

        assert_eq!(
            hit.context_before,
            vec![
                (1, "line 1".to_string()),
                (2, "line 2".to_string()),
            ]
        );

        assert_eq!(
            hit.context_after,
            vec![
                (4, "line 4".to_string()),
                (5, "line 5".to_string()),
            ]
        );
    }

    #[test]
    fn plain_text_can_be_case_insensitive() {
        let mut options = opts("ggsave");
        options.case_insensitive = true;

        let hits = search_text(
            Path::new("script.R"),
            "GGSAVE(\"plot.png\")",
            &options,
        )
        .unwrap();

        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn regex_search_works() {
        let mut options = opts(r"ggsave|png\(");
        options.use_regex = true;

        let text = "\
png(\"plot.png\")
dev.off()
ggsave(\"plot.svg\")";

        let hits = search_text(
            Path::new("script.R"),
            text,
            &options,
        )
        .unwrap();

        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].line_number, 1);
        assert_eq!(hits[1].line_number, 3);
    }

    #[test]
    fn invalid_regex_returns_error() {
        let mut options = opts("(");
        options.use_regex = true;

        let result = search_text(
            Path::new("script.R"),
            "anything",
            &options,
        );

        assert!(result.is_err());
    }

    #[test]
    fn empty_query_returns_no_hits() {
        let result = search_text(
            Path::new("script.R"),
            "ggsave(\"plot.png\")",
            &opts(""),
        )
        .unwrap();

        assert!(result.is_empty());
    }

    #[test]
    fn context_at_file_start_does_not_underflow() {
        let hits = search_text(
            Path::new("script.R"),
            "match\nline2\nline3",
            &opts("match"),
        )
        .unwrap();

        assert_eq!(hits.len(), 1);
        assert!(hits[0].context_before.is_empty());
        assert_eq!(hits[0].context_after.len(), 2);
    }

    #[test]
    fn display_string_contains_path_line_and_match() {
        let hit = SearchHit {
            path: PathBuf::from("analysis/script.R"),
            line_number: 12,
            matched_line: "png(\"x.png\")".to_string(),
            context_before: vec![],
            context_after: vec![],
        };

        assert_eq!(
            hit.to_display_string(),
            "analysis/script.R:12: png(\"x.png\")"
        );
    }
}