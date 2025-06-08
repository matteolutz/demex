use serde::{Deserialize, Serialize};

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
            _ => todo!(),
        }
    }
}
