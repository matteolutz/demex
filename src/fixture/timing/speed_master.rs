use std::time;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Copy, Clone)]
pub struct SpeedMasterValue {
    bpm: f32,
    #[serde(default, skip_serializing, skip_deserializing)]
    interval: Option<time::Instant>,
}

impl SpeedMasterValue {
    pub fn new(bpm: f32) -> Self {
        Self {
            bpm,
            interval: Some(time::Instant::now()),
        }
    }

    pub fn bpm(&self) -> f32 {
        self.bpm
    }

    pub fn set_bpm(&mut self, bpm: f32) {
        self.bpm = bpm;
    }

    pub fn secs_per_beat(&self) -> f32 {
        60.0 / self.bpm
    }

    pub fn interval(&self) -> Option<time::Instant> {
        self.interval
    }

    pub fn tap(&mut self, interval: time::Instant) {
        if let Some(last_tap) = self.interval {
            let duration = interval.duration_since(last_tap);
            let bpm = 60.0 / duration.as_secs_f32();
            self.bpm = (self.bpm + bpm) / 2.0;
        }

        self.interval = Some(interval);
    }

    pub fn display_should_blink(&self) -> bool {
        // should return true if we are in the first half of a beat
        // and false if we are in the second half of a beat

        if let Some(interval) = self.interval {
            let duration = time::Instant::now().duration_since(interval).as_secs_f64()
                % self.secs_per_beat() as f64;

            duration < self.secs_per_beat() as f64 / 2.0
        } else {
            false
        }
    }
}

impl Default for SpeedMasterValue {
    fn default() -> Self {
        Self {
            bpm: 120.0,
            interval: None,
        }
    }
}
