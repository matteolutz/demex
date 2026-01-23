use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum FixtureLayoutDecoration {
    Label {
        pos: emath::Pos2,
        text: String,
        font_size: f32,
    },
    Rect {
        min: emath::Pos2,
        max: emath::Pos2,
        stroke_width: f32,
    },
}
