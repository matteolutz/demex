use serde::{Deserialize, Serialize};

use crate::utils::ease::{ease_in_out_quad, ease_in_quad, ease_out_quad};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum KeyframeEffectKeyframeCurve {
    #[default]
    Linear,

    Snap,
    EaseIn,
    EaseOut,
    EaseInOut,
}

impl KeyframeEffectKeyframeCurve {
    pub fn value(&self, t: f32) -> f32 {
        match self {
            Self::Linear => t.clamp(0.0, 1.0),
            Self::Snap => {
                if t >= 1.0 {
                    1.0
                } else {
                    0.0
                }
            }
            Self::EaseIn => ease_in_quad(t.clamp(0.0, 1.0)),
            Self::EaseOut => ease_out_quad(t.clamp(0.0, 1.0)),
            Self::EaseInOut => ease_in_out_quad(t.clamp(0.0, 1.0)),
        }
    }
}
