//! Lightweight file/example preview widgets.
//!
//! This module currently previews only bucket example paths.
//! Later this can grow into actual file preview support for text,
//! notebooks, images, and metadata.

use eframe::egui;

/// Show example paths for a bucket.
pub fn show_examples(
    ui: &mut egui::Ui,
    examples: &[String],
) -> Option<String> {
    if examples.is_empty() {
        ui.weak("No examples available.");
        return None;
    }

    let mut selected = None;

    egui::ScrollArea::vertical()
        .max_height(180.0)
        .auto_shrink([false, true])
        .show(ui, |ui| {
            for example in examples {
                if ui.link(example).clicked() {
                    selected = Some(example.clone());
                }
            }
        });

    selected
}


#[cfg(test)]
mod tests {
    fn first_three_examples(
        examples: &[String],
    ) -> Vec<String> {
        examples
            .iter()
            .take(3)
            .cloned()
            .collect()
    }

    #[test]
    fn first_three_examples_limits_output() {
        let examples = vec![
            "a.png".to_string(),
            "b.png".to_string(),
            "c.png".to_string(),
            "d.png".to_string(),
        ];

        let selected =
            first_three_examples(&examples);

        assert_eq!(
            selected,
            vec![
                "a.png".to_string(),
                "b.png".to_string(),
                "c.png".to_string(),
            ]
        );
    }

    #[test]
    fn first_three_examples_handles_short_input() {
        let examples = vec![
            "a.png".to_string(),
        ];

        let selected =
            first_three_examples(&examples);

        assert_eq!(
            selected,
            vec!["a.png".to_string()]
        );
    }
}
