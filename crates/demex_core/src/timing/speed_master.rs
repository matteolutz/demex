use std::time;

use serde::{Deserialize, Serialize};

use crate::effect::speed::EffectSpeedScale;

use super::tap::TapChain;

/// This is the factor by which the current beat
/// is lerped towards the nearest full beat when the tap chain is active.
const BEAT_LERP_FACTOR: f32 = 0.3;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SpeedMasterValue {
    bpm: f32,

    #[serde(default, skip_serializing, skip_deserializing)]
    tap_chain: TapChain,

    #[serde(default, skip_serializing, skip_deserializing)]
    last_update: Option<time::Instant>,

    /// The current beat for this speed master.
    /// This will be calculated from the last update
    #[serde(default, skip_serializing, skip_deserializing)]
    current_beat: f32,
}

impl SpeedMasterValue {
    pub fn new(bpm: f32) -> Self {
        Self {
            bpm,
            tap_chain: TapChain::new(10),
            last_update: None,
            current_beat: 0.0,
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

    pub fn bps(&self) -> f32 {
        self.bpm / 60.0
    }

    pub fn secs_per_beat(&self) -> f32 {
        60.0 / self.bpm
    }

    pub fn interval(&self) -> Option<time::Instant> {
        self.tap_chain.last_tap()
    }

    pub fn current_beat(&self) -> f32 {
        self.current_beat
    }

    pub fn current_phase(&self, scale: EffectSpeedScale) -> f32 {
        // return a phase from 0.0 to 2pi based on the scale.
        // so if scale.scale_value() is 1.0, then a single beat should
        // correspond to a phase of 2pi (one full cycle)

        let scale_value = scale.scale_value();
        (self.current_beat * scale_value) % 2.0 * std::f32::consts::PI
    }

    pub fn update(&mut self) {
        let now = time::Instant::now();

        let Some(last_update) = self.last_update else {
            // this is the first update, so just set the last update and return
            self.last_update = Some(now);
            return;
        };

        let elapsed = now.duration_since(last_update);

        let elapsed_beats = elapsed.as_secs_f32() * self.bps();

        self.current_beat += elapsed_beats;

        self.last_update = Some(now);
    }

    pub fn tap(&mut self, instant: time::Instant) -> Option<f32> {
        let bpm_before = self.bpm;
        self.bpm = self.tap_chain.tap(instant, self.bpm);

        // self.current_beat = 0.0; // TODO: find a better solution for this
        // slowly lerp the current beat to the nearest full beat
        let target_beat = self.current_beat.round();
        let lerp_factor = BEAT_LERP_FACTOR;
        self.current_beat = self.current_beat + (target_beat - self.current_beat) * lerp_factor;

        (self.bpm != bpm_before).then_some(self.bpm)
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
            last_update: None,
            current_beat: 0.0,
        }
    }
}
