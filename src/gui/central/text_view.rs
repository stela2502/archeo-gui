//! Central text-file tab viewer.
//!
//! This module renders open text files as tabs. Each tab stores its own
//! navigation state (focused line, requested scroll target, etc.).
//!
//! The actual text-file state and helper logic lives in `OpenTextFile`
//! inside `state.rs`.

use eframe::egui;

use crate::gui::state::{GuiState};
use crate::gui::central::open_text_file::OpenTextFile;


/// Render one text file
pub fn show_file(
    ui: &mut egui::Ui,
    open_file: &mut OpenTextFile,
) {
    ui.heading(open_file.tab_label());

    ui.add_space(6.0);

    draw_file_header(ui, open_file);

    ui.separator();

    draw_text(ui, open_file);
}

/// Draw the text-file tab bar.
fn draw_tab_bar(
    ui: &mut egui::Ui,
    state: &mut GuiState,
) {
    let mut close_index: Option<usize> = None;

    ui.horizontal_wrapped(|ui| {
        for index in 0..state.tabs.len() {
            let tab =
                &state.tabs[index];

            let selected =
                state.active_tab
                    == Some(index);

            // THIS is where the impl method is used:
            let label = tab.tab_label();

            if ui
                .selectable_label(selected, label)
                .clicked()
            {
                state.active_tab =
                    Some(index);
            }

            if ui.small_button("×").clicked() {
                close_index = Some(index);
            }
        }
    });

    if let Some(index) = close_index {
        close_tab(state, index);
    }
}

/// Close a tab and keep the active index valid.
fn close_tab(
    state: &mut GuiState,
    index: usize,
) {
    if index >= state.tabs.len() {
        return;
    }

    state.tabs.remove(index);

    state.active_tab =
        match state.tabs.len() {
            0 => None,

            len => {
                let previous =
                    state.active_tab
                        .unwrap_or(0);

                if previous == index {
                    Some(index.min(len - 1))
                } else if previous > index {
                    Some(previous - 1)
                } else {
                    Some(previous)
                }
            }
        };
}

/// Draw metadata for the active file.
fn draw_file_header(
    ui: &mut egui::Ui,
    open_file: &OpenTextFile,
) {
    ui.horizontal_wrapped(|ui| {
        ui.monospace(
            open_file.rel_path.display().to_string(),
        );

        if let Some(language) =
            &open_file.language_hint
        {
            ui.separator();

            ui.label(format!(
                "language: {language}"
            ));
        }

        if let Some(line) =
            open_file.focused_line
        {
            ui.separator();

            ui.label(format!(
                "focused line: {line}"
            ));
        }

        ui.separator();

        ui.label(format!(
            "lines: {}",
            open_file.line_count()
        ));
    });
}

/// Draw file text with line numbers.
///
/// The requested line is centered only once.
/// Afterwards the file behaves like a normal editor view.
fn draw_text(
    ui: &mut egui::Ui,
    open_file: &mut OpenTextFile,
) {
    egui::ScrollArea::both()
        .id_salt((
            "text_view",
            open_file.rel_path.clone(),
        ))
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for (index, line) in
                open_file.content.lines().enumerate()
            {
                let line_no = index + 1;

                let highlighted =
                    open_file
                        .is_highlighted_line(line_no);

                let response = draw_line(
                    ui,
                    line_no,
                    line,
                    highlighted,
                );

                if open_file.should_scroll_to(line_no)
                {
                    response.scroll_to_me(Some(
                        egui::Align::Center,
                    ));

                    open_file.mark_scroll_consumed();

                    open_file.focus_line(line_no);
                }

                if response.clicked() {
                    open_file.focus_line(line_no);
                }
            }
        });
}

/// Draw one numbered text line.
fn draw_line(
    ui: &mut egui::Ui,
    line_no: usize,
    line: &str,
    highlighted: bool,
) -> egui::Response {
    ui.horizontal(|ui| {
        if highlighted {
            ui.colored_label(
                egui::Color32::YELLOW,
                format!("{line_no:>5}"),
            );

            ui.colored_label(
                egui::Color32::YELLOW,
                egui::RichText::new(line)
                    .monospace(),
            );
        } else {
            ui.weak(format!("{line_no:>5}"));

            ui.monospace(line);
        }
    })
    .response
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::gui::state::{
        GuiState,
        OpenTextFile,
    };

    fn make_file(name: &str) -> OpenTextFile {
        OpenTextFile::new(
            PathBuf::from(format!(
                "/tmp/{name}"
            )),
            PathBuf::from(name),
            "a\nb\nc".to_string(),
            Some("text".to_string()),
            Some(2),
        )
    }

    #[test]
    fn close_tab_removes_tab() {
        let mut state = GuiState::default();

        state
            .workspace
            .text_tabs
            .push(make_file("a.txt"));

        state
            .workspace
            .text_tabs
            .push(make_file("b.txt"));

        state.active_tab =
            Some(0);

        close_tab(&mut state, 0);

        assert_eq!(
            state.tabs.len(),
            1
        );

        assert_eq!(
            state.tabs[0]
                .rel_path,
            PathBuf::from("b.txt")
        );
    }

    #[test]
    fn close_last_tab_clears_active_tab() {
        let mut state = GuiState::default();

        state
            .workspace
            .text_tabs
            .push(make_file("a.txt"));

        state.active_tab =
            Some(0);

        close_tab(&mut state, 0);

        assert!(
            state.tabs.is_empty()
        );

        assert_eq!(
            state.active_tab,
            None
        );
    }

    #[test]
    fn open_text_file_tracks_focus_line() {
        let mut file =
            make_file("script.R");

        assert_eq!(
            file.focused_line,
            Some(2)
        );

        file.focus_line(3);

        assert_eq!(
            file.focused_line,
            Some(3)
        );
    }

    #[test]
    fn scroll_request_is_consumed_once() {
        let mut file =
            make_file("script.R");

        assert!(file.should_scroll_to(2));

        file.mark_scroll_consumed();

        assert!(!file.should_scroll_to(2));
    }

    #[test]
    fn tab_label_uses_file_name() {
        let file =
            make_file("analysis/script.R");

        assert_eq!(
            file.tab_label(),
            "script.R"
        );
    }
}

