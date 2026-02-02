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
#[derive(Default, Debug, Clone)]
pub struct WaveSegment {
    pub starting_point: f32,
    pub values: Vec<f32>,
}
