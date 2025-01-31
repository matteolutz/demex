use core::fmt;

use egui_probe::EguiProbe;
use serde::{Deserialize, Serialize};

use super::channel::error::FixtureChannelError;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, EguiProbe)]
pub enum FixtureChannelEffect {
    SingleSine { a: f32, b: f32, c: f32, d: f32 },
}

impl Default for FixtureChannelEffect {
    fn default() -> Self {
        Self::SingleSine {
            a: 0.0,
            b: 0.0,
            c: 0.0,
            d: 0.0,
        }
    }
}

impl FixtureChannelEffect {
    pub fn as_single(&self, t: f32) -> Result<f32, FixtureChannelError> {
        match self {
            Self::SingleSine { a, b, c, d } => Ok((a * f32::sin(b * t + c) + d).clamp(0.0, 1.0)),
            #[allow(unreachable_patterns)]
            _ => Err(FixtureChannelError::FixtureChannelValueWrongVariant(
                "Single".to_owned(),
            )),
        }
    }

    pub fn as_pair(&self, _t: f32) -> Result<[f32; 2], FixtureChannelError> {
        todo!("effect as_pair")
    }

    pub fn as_quadruple(&self, _t: f32) -> Result<[f32; 4], FixtureChannelError> {
        todo!("effect as_quadruple")
    }
}

impl fmt::Display for FixtureChannelEffect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SingleSine { a, b, c, d } => write!(f, "{} * sin({}t + {}) + {}", a, b, c, d),
        }
    }
}
