use std::{collections::VecDeque, time};

use itertools::Itertools;

const MAX_BEAT_LENGTH: f64 = 60.0 / 30.0;
const CHAIN_RESET_BEATS: usize = 2;
const MIN_TAPS_FOR_NEW_BPM: usize = 2;

#[derive(Debug, Clone)]
pub struct TapChain {
    taps: VecDeque<time::Instant>,
}

impl Default for TapChain {
    fn default() -> Self {
        Self::new(10)
    }
}

impl TapChain {
    pub fn new(max_taps: usize) -> Self {
        Self {
            taps: VecDeque::with_capacity(max_taps),
        }
    }

    fn chain_active(&self, instant: time::Instant, last_bpm: f32) -> bool {
        if let Some(last_tap) = self.taps.back() {
            *last_tap + time::Duration::from_secs_f64(MAX_BEAT_LENGTH) > instant
                && *last_tap
                    + (time::Duration::from_secs_f64(
                        self.beat_interval(last_bpm) * CHAIN_RESET_BEATS as f64,
                    ))
                    > instant
        } else {
            true
        }
    }

    pub fn last_tap(&self) -> Option<time::Instant> {
        self.taps.back().copied()
    }

    fn beat_interval(&self, last_bpm: f32) -> f64 {
        60.0 / last_bpm as f64
    }

    fn average_bpm(&self) -> f32 {
        self.taps
            .iter()
            .tuple_windows()
            .map(|(a, b)| {
                let duration = b.duration_since(*a);
                (60.0 / duration.as_secs_f64()) as f32
            })
            .sum::<f32>()
            / (self.taps.len() as f64 - 1.0) as f32
    }

    /// Starts a new tap chain if the last tap is older than `MAX_BEAT_LENGTH`.
    /// Otherwise, adds the tap to the chain and updates the BPM.
    /// Returns the new (or unchanged) BPM.
    pub fn tap(&mut self, instant: time::Instant, mut last_bpm: f32) -> f32 {
        if !self.chain_active(instant, last_bpm) {
            self.taps.clear();
        }

        if !self.taps.is_empty() {
            if self.taps.len() == self.taps.capacity() {
                self.taps.pop_front();
            }

            self.taps.push_back(instant);

            if self.taps.len() >= MIN_TAPS_FOR_NEW_BPM {
                last_bpm = self.average_bpm();
            }
        } else {
            self.taps.push_back(instant);
        }

        last_bpm
    }
}
