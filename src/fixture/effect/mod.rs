use std::{f32, time};

use egui_probe::EguiProbe;
use serde::{Deserialize, Serialize};

use super::channel::error::FixtureChannelError;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, EguiProbe)]
pub enum FixtureChannelEffect {
    SingleSine {
        a: f32,
        b: f32,
        c: f32,
        d: f32,
    },
    PairFigureEight {
        speed: f32,
        center_a: f32,
        center_b: f32,
    },
    QuadrupleHueRotate {
        speed: f32,
    },
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
    pub fn as_single(&self, started: &Option<time::Instant>) -> Result<f32, FixtureChannelError> {
        let t = started
            .map(|started| started.elapsed().as_secs_f32())
            .unwrap_or(0.0);

        match self {
            Self::SingleSine { a, b, c, d } => Ok((a * f32::sin(b * t + c) + d).clamp(0.0, 1.0)),
            _ => Err(FixtureChannelError::FixtureChannelValueWrongVariant(
                "Single".to_owned(),
            )),
        }
    }

    pub fn as_pair(
        &self,
        started: &Option<time::Instant>,
    ) -> Result<[f32; 2], FixtureChannelError> {
        let t = started
            .map(|started| started.elapsed().as_secs_f32())
            .unwrap_or(0.0);

        match self {
            Self::PairFigureEight {
                speed,
                center_a,
                center_b,
            } => {
                let c = center_a.min(1.0 - center_a);
                let d = center_b.min(1.0 - center_b);

                let a = center_a + c * f32::sin(speed * t);
                let b = center_b + d * f32::sin(2.0 * speed * t + (f32::consts::FRAC_PI_2));

                Ok([a, b])
            }
            _ => Err(FixtureChannelError::FixtureChannelValueWrongVariant(
                "Pair".to_owned(),
            )),
        }
    }

    pub fn as_quadruple(
        &self,
        started: &Option<time::Instant>,
    ) -> Result<[f32; 4], FixtureChannelError> {
        let t = started
            .map(|started| started.elapsed().as_secs_f32())
            .unwrap_or(0.0);

        match self {
            Self::QuadrupleHueRotate { speed } => {
                let r = f32::sin(speed * t);
                let g = f32::sin(speed * t + (f32::consts::FRAC_PI_2));
                let b = f32::sin(speed * t + (f32::consts::PI));

                Ok([r, g, b, 1.0])
            }
            _ => Err(FixtureChannelError::FixtureChannelValueWrongVariant(
                "Quadruple".to_owned(),
            )),
        }
    }

    pub fn to_string(&self, _started: &Option<time::Instant>) -> String {
        match self {
            Self::SingleSine { a, b, c, d } => format!("{} * sin({}t + {}) + {}", a, b, c, d),
            Self::PairFigureEight {
                speed,
                center_a,
                center_b,
            } => format!("PairFigureEight({}, {}, {})", speed, center_a, center_b),
            Self::QuadrupleHueRotate { speed } => format!("QuadrupleHueRotate({})", speed),
        }
    }
}
