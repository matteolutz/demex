use crate::color_gel;

use super::ColorGel;

pub const DEMEX_COLOR_GELS: &[ColorGel] = &[
    color_gel!("White", [1.0, 1.0, 1.0]),
    color_gel!("Warm White", [0.937, 0.922, 0.847]),
    color_gel!("Red", [1.0, 0.0, 0.0]),
    color_gel!("Green", [0.0, 1.0, 0.0]),
    color_gel!("Blue", [0.0, 0.0, 1.0]),
    color_gel!("Cyan", [0.0, 1.0, 1.0]),
    color_gel!("Magenta", [1.0, 0.0, 1.0]),
    color_gel!("Yellow", [1.0, 1.0, 0.0]),
];
