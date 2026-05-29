use eframe::egui;

use crate::gui::search::text_file::SearchHit;

pub fn show(
    ui: &mut egui::Ui,
    query: &str,
    hits: &[SearchHit],
) -> Option<SearchHit> {
    ui.heading(format!("Search: {query}"));

    if hits.is_empty() {
        ui.weak("No hits.");
        return None;
    }

    let mut open_hit = None;

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for hit in hits {
                if draw_hit(ui, hit).clicked() {
                    open_hit = Some(hit.clone());
                }

                ui.add_space(6.0);
            }
        });

    open_hit
}

fn draw_hit(
    ui: &mut egui::Ui,
    hit: &SearchHit,
) -> egui::Response {
    ui.group(|ui| {
        ui.monospace(format!(
            "{}:{}",
            hit.path.display(),
            hit.line_number
        ));

        for (line_no, line) in &hit.context_before {
            draw_context_line(ui, *line_no, line);
        }

        draw_match_line(ui, hit.line_number, &hit.matched_line);

        for (line_no, line) in &hit.context_after {
            draw_context_line(ui, *line_no, line);
        }
    })
    .response
}

fn draw_context_line(ui: &mut egui::Ui, line_no: usize, line: &str) {
    ui.horizontal(|ui| {
        ui.weak(format!("{line_no:>5}"));
        ui.monospace(line);
    });
}

fn draw_match_line(ui: &mut egui::Ui, line_no: usize, line: &str) {
    ui.horizontal(|ui| {
        ui.colored_label(egui::Color32::YELLOW, format!("{line_no:>5}"));
        ui.colored_label(
            egui::Color32::YELLOW,
            egui::RichText::new(line).monospace(),
        );
    });
}