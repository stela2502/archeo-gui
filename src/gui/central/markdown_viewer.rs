use eframe::egui;
use egui::text::{LayoutJob, TextFormat};
use egui::{Color32, FontId};

use crate::gui::state::MarkdownDraft;

pub fn show(ui: &mut egui::Ui, draft: &mut MarkdownDraft) {
    ui.horizontal(|ui| {
        ui.heading(&draft.title);

        if draft.dirty {
            ui.colored_label(Color32::YELLOW, "modified");
        }

        if ui.button("Save").clicked() {
            if let Err(err) = draft.save() {
                draft.last_error = Some(err.to_string());
            }
        }

        if ui.button("Save as...").clicked() {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Markdown", &["md"])
                .set_file_name("report.md")
                .save_file()
            {
                if let Err(err) = draft.save_as(path) {
                    draft.last_error = Some(err.to_string());
                }
            }
        }
    });

    if let Some(path) = &draft.path {
        ui.weak(path.display().to_string());
    } else {
        ui.weak("Unsaved Markdown draft");
    }

    if let Some(err) = &draft.last_error {
        ui.colored_label(Color32::LIGHT_RED, err);
    }

    ui.separator();

    let mut layouter = |ui: &egui::Ui, text: &dyn egui::TextBuffer, wrap_width: f32| {
        let mut job = highlight_markdown(text.as_str(), wrap_width);
        ui.fonts(|fonts| fonts.layout_job(job))
    };

    let response = egui::TextEdit::multiline(&mut draft.content)
        .font(egui::TextStyle::Monospace)
        .desired_width(f32::INFINITY)
        .desired_rows(35)
        .layouter(&mut layouter)
        .show(ui);

    if response.response.changed() {
        draft.dirty = true;
    }
}

fn highlight_markdown(text: &str, wrap_width: f32) -> LayoutJob {
    let mut job = LayoutJob::default();
    job.wrap.max_width = wrap_width;

    let normal = fmt(Color32::LIGHT_GRAY);
    let heading = fmt(Color32::LIGHT_BLUE);
    let code = fmt(Color32::LIGHT_GREEN);
    let quote = fmt(Color32::GRAY);
    let list = fmt(Color32::YELLOW);

    for line in text.lines() {
        let trimmed = line.trim_start();

        let format = if trimmed.starts_with('#') {
            heading.clone()
        } else if trimmed.starts_with("```") || trimmed.starts_with('`') {
            code.clone()
        } else if trimmed.starts_with('>') {
            quote.clone()
        } else if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
            list.clone()
        } else {
            normal.clone()
        };

        job.append(line, 0.0, format);
        job.append("\n", 0.0, normal.clone());
    }

    job
}

fn fmt(color: Color32) -> TextFormat {
    TextFormat {
        font_id: FontId::monospace(13.0),
        color,
        ..Default::default()
    }
}