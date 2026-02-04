use std::rc::Rc;

use gpui::{Pixels, Point};

#[derive(Default, Debug)]
pub struct Wave {
    pub segments: Vec<WaveSegment>,
}

impl Wave {
    pub fn add_segment(&mut self, segment: WaveSegment) {
        self.segments.push(segment);
    }
}

/// A single wave segment can split into multiple lines (i.e. if an effect has different values for different fixtures)
#[derive(Debug, Clone)]
pub struct WaveSegment {
    pub starting_point: f32,
    pub values: Vec<f32>,
    pub easing_functions: Rc<dyn WaveEasingFunction>,
}

#[derive(Debug, Copy, Clone)]
pub enum WaveEasingMode {
    Linear,
    Snap,
    Cubic(Point<Pixels>, Point<Pixels>),
}

impl From<(Point<Pixels>, Point<Pixels>)> for WaveEasingMode {
    fn from(points: (Point<Pixels>, Point<Pixels>)) -> Self {
        Self::Cubic(points.0, points.1)
    }
}

pub trait WaveEasingFunction: 'static + std::fmt::Debug {
    fn get_easing_mode(&self, from: Point<Pixels>, to: Point<Pixels>) -> WaveEasingMode;
}
