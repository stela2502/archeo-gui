use std::path::PathBuf;


pub struct OpenTextFile {
    pub path: PathBuf,
    pub rel_path: PathBuf,
    pub content: String,
    pub language_hint: Option<String>,

    /// Line that should be brought into view once.
    pub requested_line: Option<usize>,

    /// Last line the user intentionally centered/jumped to.
    pub focused_line: Option<usize>,

    /// Has `requested_line` already been consumed by the UI?
    pub scroll_request_consumed: bool,
}

impl OpenTextFile {
    pub fn new(
        path: PathBuf,
        rel_path: PathBuf,
        content: String,
        language_hint: Option<String>,
        requested_line: Option<usize>,
    ) -> Self {
        Self {
            path,
            rel_path,
            content,
            language_hint,
            requested_line,
            focused_line: requested_line,
            scroll_request_consumed: false,
        }
    }

    pub fn tab_label(&self) -> String {
        self.rel_path
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| self.rel_path.display().to_string())
    }

    pub fn should_scroll_to(&self, line_no: usize) -> bool {
        self.requested_line == Some(line_no)
            && !self.scroll_request_consumed
    }

    pub fn mark_scroll_consumed(&mut self) {
        self.scroll_request_consumed = true;
    }

    pub fn focus_line(&mut self, line_no: usize) {
        self.focused_line = Some(line_no);
    }

    pub fn is_highlighted_line(&self, line_no: usize) -> bool {
        self.requested_line == Some(line_no)
            || self.focused_line == Some(line_no)
    }

    pub fn line_count(&self) -> usize {
        self.content.lines().count()
    }
}



