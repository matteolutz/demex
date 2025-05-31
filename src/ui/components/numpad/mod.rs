use crate::ui::utils::painter::painter_layout_centered;

const WIDTH: f32 = 200.0;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum NumpadAction {
    Insert(char),
    Pop,
}

#[derive(Debug, Clone)]
pub enum NumpadResult {
    Value(f32),
    ChannelSet(String),
}

impl NumpadResult {
    pub fn from_str(value: &str) -> Option<Self> {
        if let Ok(num) = value.parse::<f32>() {
            return Some(NumpadResult::Value(num));
        }

        let channel_set_re = regex::Regex::new(r#"\s*"(.+)"\s*"#).unwrap();
        if let Some(captures) = channel_set_re.captures(value) {
            return Some(NumpadResult::ChannelSet(
                captures.get(1).unwrap().as_str().to_owned(),
            ));
        }

        None
    }
}

impl std::fmt::Display for NumpadAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NumpadAction::Insert(c) => write!(f, "{}", c),
            NumpadAction::Pop => write!(f, "Bksp"),
        }
    }
}

pub fn numpad_ui(ui: &mut egui::Ui, value: &mut String) {
    ui.vertical(|ui| {
        egui::TextEdit::singleline(value)
            .desired_width(WIDTH)
            .show(ui);

        let num_cols = 3;
        let num_rows = 4;

        let (response, painter) = ui.allocate_painter(
            egui::vec2(WIDTH, WIDTH * (num_rows as f32 / num_cols as f32)),
            egui::Sense::click(),
        );

        let width = response.rect.width() / num_cols as f32;
        let height = response.rect.height() / num_rows as f32;

        for col in 0..num_cols {
            for row in 0..num_rows {
                let rect = egui::Rect::from_min_size(
                    response.rect.left_top() + egui::vec2(col as f32 * width, row as f32 * height),
                    egui::vec2(width, height),
                )
                .shrink(1.0);

                let action: NumpadAction = match (col, row) {
                    (0, 3) => {
                        // Button for .
                        NumpadAction::Insert('.')
                    }
                    (2, 3) => NumpadAction::Pop,
                    (col, row) => {
                        let num: u8 = if (col, row) == (1, 3) {
                            0
                        } else {
                            col + row * 3 + 1
                        };

                        NumpadAction::Insert((num + b'0') as char)
                    }
                };

                painter.rect_filled(rect, 0.0, egui::Color32::DARK_GRAY);
                painter_layout_centered(
                    &painter,
                    action.to_string(),
                    egui::FontId::proportional(12.0),
                    egui::Color32::WHITE,
                    rect,
                );

                if response
                    .hover_pos()
                    .is_some_and(|hover_pos| rect.contains(hover_pos))
                {
                    ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
                    painter.rect_stroke(
                        rect,
                        0.0,
                        (1.0, egui::Color32::WHITE),
                        egui::StrokeKind::Middle,
                    );
                }

                if response
                    .interact_pointer_pos()
                    .is_some_and(|interact_pos| response.clicked() && rect.contains(interact_pos))
                {
                    match action {
                        NumpadAction::Pop => {
                            value.pop();
                        }
                        NumpadAction::Insert(char) => {
                            value.push(char);
                        }
                    }
                }
            }
        }
    });
}
