use std::time;

use serde::{Deserialize, Serialize};

use super::tap::TapChain;

#[derive(Serialize, Deserialize, Clone)]
pub struct SpeedMasterValue {
    bpm: f32,

    #[serde(default, skip_serializing, skip_deserializing)]
    tap_chain: TapChain,
}

impl SpeedMasterValue {
    pub fn new(bpm: f32) -> Self {
        Self {
            bpm,
            tap_chain: TapChain::new(10),
        }
    }

    pub fn bpm(&self) -> f32 {
        self.bpm
    }

    pub fn bpm_mut(&mut self) -> &mut f32 {
        &mut self.bpm
    }

    pub fn set_bpm(&mut self, bpm: f32) {
        self.bpm = bpm;
    }

    pub fn secs_per_beat(&self) -> f32 {
        60.0 / self.bpm
    }

    pub fn interval(&self) -> Option<time::Instant> {
        self.tap_chain.last_tap()
    }

    pub fn tap(&mut self, instant: time::Instant) {
        self.bpm = self.tap_chain.tap(instant, self.bpm);
    }

    pub fn on_beat(&self) -> bool {
        // should return true if we are in the first half of a beat
        // and false if we are in the second half of a beat

        if let Some(interval) = self.tap_chain.last_tap() {
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
            tap_chain: TapChain::default(),
        }
    }
}
