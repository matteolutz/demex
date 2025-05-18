use std::f32;

use bezier::cubic_bezier;
use itertools::Itertools;
use segment::WaveSegment;
use serde::{Deserialize, Serialize};
use wave_type::WaveType;

pub mod bezier;
pub mod segment;
pub mod wave_type;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ui", derive(egui_probe::EguiProbe))]
pub struct Effect2Wave {
    segments: Vec<WaveSegment>,
    wave_type: WaveType,
}

impl Default for Effect2Wave {
    fn default() -> Self {
        Self {
            segments: vec![
                WaveSegment::new(
                    egui::pos2(0.0, 0.0),
                    [egui::pos2(0.0, 0.0), egui::pos2(0.2, 0.0)],
                ),
                WaveSegment::new(
                    egui::pos2(0.5, 1.0),
                    [egui::pos2(0.3, 1.0), egui::pos2(0.7, 1.0)],
                ),
                WaveSegment::new(
                    egui::pos2(1.0, 0.0),
                    [egui::pos2(0.8, 0.0), egui::pos2(1.0, 0.0)],
                ),
            ],
            wave_type: WaveType::Bezier,
        }
    }
}

impl Effect2Wave {
    pub fn segments(&self) -> &[WaveSegment] {
        &self.segments
    }

    pub fn segments_mut(&mut self) -> &mut Vec<WaveSegment> {
        &mut self.segments
    }

    pub fn wave_type(&self) -> WaveType {
        self.wave_type
    }

    pub fn wave_type_mut(&mut self) -> &mut WaveType {
        &mut self.wave_type
    }

    pub fn segment_tuples<'a>(
        &'a self,
        end_segment: &'a WaveSegment,
    ) -> impl Iterator<Item = (&'a WaveSegment, &'a WaveSegment)> {
        if self.segments().len() < 2 {
            [&self.segments[0], end_segment]
                .into_iter()
                .sorted_by(|a, b| a.start_pos().x.partial_cmp(&b.start_pos().x).unwrap())
                .tuple_windows::<(_, _)>()
        } else {
            self.segments
                .iter()
                .sorted_by(|a, b| a.start_pos().x.partial_cmp(&b.start_pos().x).unwrap())
                .tuple_windows::<(_, _)>()
        }
    }

    pub fn insert_segment(&mut self, pos: emath::Pos2) {
        self.segments.push(WaveSegment::from_start_pos(
            pos.clamp(emath::Pos2::ZERO, emath::pos2(1.0, 1.0)),
        ));
    }

    // TODO: optimize this
    pub fn value(&self, time: f32) -> f32 {
        // t is the actual, we have to convert it, to map 0-2pi to 0-1
        let t = (time % (2.0 * f32::consts::PI)) / (2.0 * f32::consts::PI);

        for (a, b) in self.segment_tuples(&WaveSegment::from_start_pos(emath::pos2(1.0, 0.0))) {
            if t < a.start_pos().x || t > b.start_pos().x {
                continue;
            }

            let points = [
                a.start_pos(),
                a.control_points()[1],
                b.control_points()[0],
                b.start_pos(),
            ];

            let t = (t - points[0].x) / (points[3].x - points[0].x);

            let val = match self.wave_type {
                WaveType::Triangle => points[0].y + (points[3].y - points[0].y) * t,
                WaveType::Square => {
                    if t < 0.5 {
                        points[0].y
                    } else {
                        points[3].y
                    }
                }
                WaveType::Bezier => cubic_bezier(points, t).y,
            };

            return val.clamp(0.0, 1.0);
        }

        /*
        let end_point = ;

        for (idx, seg) in self
            .segments
            .iter()
            .sorted_by(|a, b| a.start_pos().x.partial_cmp(&b.start_pos().x).unwrap())
            .enumerate()
        {
            if seg.start_pos().x < t {
                continue;
            }

            if idx == 0 {
                return 0.0;
            }

            let prev_seg = &self.segments[idx - 1];

            let points = [
                prev_seg.start_pos(),
                prev_seg.control_points()[0],
                prev_seg.control_points()[1],
                seg.start_pos(),
            ];

            let old_t = t;
            let t = (t - points[0].x) / (points[3].x - points[0].x);
            log::debug!("old_t: {}, t: {}", old_t, t);

            if t < 0.0 {
                return 0.0;
            }

            return match self.wave_type {
                WaveType::Triangle => points[0].y + (points[3].y - points[0].y) * t,
                WaveType::Square => {
                    if t < 0.5 {
                        points[0].y
                    } else {
                        points[3].y
                    }
                }
                WaveType::Bezier => cubic_bezier(points, t),
            };
        }
        */

        0.0
    }
}
