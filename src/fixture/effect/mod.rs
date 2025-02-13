use std::{f32, time};

use egui_probe::EguiProbe;
use serde::{Deserialize, Serialize};

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
