use egui_probe::EguiProbe;
use serde::{Deserialize, Serialize};

use crate::utils::math::{
    snap_both_start, zero_one_sin, zero_one_sin_snap_in_end, zero_one_sin_snap_in_start,
    zero_one_sin_snap_out_end, zero_one_sin_snap_out_start,
};

#[derive(Debug, Serialize, Deserialize, Clone, EguiProbe, Default)]
pub enum SineVariant {
    #[default]
    Default,

    SnapInStart,
    SnapInEnd,

    SnapOutStart,
    SnapOutEnd,

    SnapBoth,
}

impl SineVariant {
    pub fn apply(&self, x: f32) -> f32 {
        match self {
            Self::Default => zero_one_sin(x),
            Self::SnapInStart => zero_one_sin_snap_in_start(x),
            Self::SnapInEnd => zero_one_sin_snap_in_end(x),
            Self::SnapOutStart => zero_one_sin_snap_out_start(x),
            Self::SnapOutEnd => zero_one_sin_snap_out_end(x),
            Self::SnapBoth => snap_both_start(x),
        }
    }
}
