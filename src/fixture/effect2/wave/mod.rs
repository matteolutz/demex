use std::f32;

use control_point::Effect2WaveControlPoint;
use egui_probe::EguiProbe;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

pub mod control_point;

#[derive(Debug, Clone, Serialize, Deserialize, EguiProbe, Default)]
pub struct Effect2Wave {
    control_points: Vec<Effect2WaveControlPoint>,
}

impl Effect2Wave {
    // TODO: optimize this
    pub fn value(&self, time: f32) -> f32 {
        // t is the actual, we have to convert it, to map 0-2pi to 0-1
        let t = (time % (2.0 * f32::consts::PI)) / (2.0 * f32::consts::PI);

        for (idx, point) in self
            .control_points
            .iter()
            .sorted_by(|a, b| a.x().partial_cmp(&b.x()).unwrap())
            .enumerate()
        {
            if point.x() < t {
                continue;
            }

            let prev_point = if idx == 0 {
                None
            } else {
                Some(&self.control_points[idx - 1])
            };

            let t = prev_point
                .map(|prev_point| (t - prev_point.x()) / (point.x() - prev_point.x()))
                .unwrap_or(1.0);
            return point.value_to_prev_point(prev_point, t);
        }

        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::{control_point::Effect2WaveControlPointWaveType, *};

    #[test]
    fn test_wave_basic() {
        let wave = Effect2Wave {
            control_points: vec![
                Effect2WaveControlPoint::new(0.0, 0.5, Effect2WaveControlPointWaveType::Linear),
                Effect2WaveControlPoint::new(0.25, 1.0, Effect2WaveControlPointWaveType::Linear),
                Effect2WaveControlPoint::new(0.75, 0.0, Effect2WaveControlPointWaveType::Linear),
                Effect2WaveControlPoint::new(1.0, 0.5, Effect2WaveControlPointWaveType::Linear),
            ],
        };

        let steps = 100;
        for i in 0..=steps {
            let t = (i as f32) / (steps as f32);
            let wave_value = wave.value(t);
            println!("w({}) = {}", t, wave_value);
        }
    }
}
