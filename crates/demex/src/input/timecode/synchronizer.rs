use std::collections::VecDeque;
use std::time::Instant;

use crate::input::{
    midi::MidiQuarterTimecodePiece,
    timecode::packet::{TimecodePacket, TimedTimecodePacket},
};

const MAX_TIME_DIFF_FOR_RESYNC_MS: u64 = 50; // Max acceptable drift before resync

#[derive(Debug, Clone)]
pub struct TimecodeSynchronizer {
    // The current estimated timecode in milliseconds
    current_estimated_millis: u64,
    // The actual system time when the internal clock started/was reset
    internal_clock_start_time: Instant,
    // To smooth out jitter and estimate average frame rate
    timecode_history: VecDeque<TimedTimecodePacket>,
}

impl Default for TimecodeSynchronizer {
    fn default() -> Self {
        Self::new()
    }
}

impl TimecodeSynchronizer {
    pub fn new() -> Self {
        TimecodeSynchronizer {
            current_estimated_millis: 0,
            internal_clock_start_time: Instant::now(), // This will be reset on first timecode
            timecode_history: VecDeque::with_capacity(5), // Keep a few for averaging
        }
    }

    pub fn process_new_quarter_frame(&mut self, frame: MidiQuarterTimecodePiece) -> bool {
        let mut last_tc = self
            .timecode_history
            .back()
            .cloned()
            .map(|tc| tc.packet)
            .unwrap_or_default();

        last_tc.update_from(frame);
        self.process_new_timecode(last_tc)
    }

    /// Call this whenever a new Timecode is received from the decoder.
    pub fn process_new_timecode(&mut self, packet: TimecodePacket) -> bool {
        let received_at = Instant::now(); // Get accurate system time NOW

        let new_timed_tc = TimedTimecodePacket {
            packet: packet.clone(),
            received_at,
        };

        let should_reset = if !self.timecode_history.is_empty()
            && self.timecode_history.back().unwrap().packet < new_timed_tc.packet
        {
            let new_tc_millis = packet.millis();

            let current_drift =
                (self.current_estimated_millis as i64 - new_tc_millis as i64).abs() as u64;

            if current_drift > MAX_TIME_DIFF_FOR_RESYNC_MS {
                // Large jump or significant drift, resync
                log::debug!(
                    "Resyncing: Estimated {}ms, Actual {}ms. Drift: {}ms",
                    self.current_estimated_millis,
                    new_tc_millis,
                    current_drift
                );
                self.current_estimated_millis = new_tc_millis;
                self.internal_clock_start_time = received_at; // Reset internal clock
            } else {
                // If it's a small jump, we can try to smooth it.
                // For now, let's just update the estimated millis.
                // More advanced: calculate average drift and apply correction.
                self.current_estimated_millis = new_tc_millis;
            }

            false
        } else {
            self.timecode_history.clear();
            // First timecode received, initialize
            self.current_estimated_millis = packet.millis();
            self.internal_clock_start_time = received_at; // Sync internal clock
            true
        };

        self.timecode_history.push_back(new_timed_tc);
        if self.timecode_history.len() > 5 {
            self.timecode_history.pop_front();
        }

        should_reset
    }

    /// Call this frequently (e.g., every few milliseconds in your main loop)
    /// to get the current estimated timecode.
    pub fn update_estimated(&mut self) -> u64 {
        if let Some(last_tc_info) = self.timecode_history.back() {
            let elapsed_since_last_tc = last_tc_info.received_at.elapsed().as_millis() as u64;
            self.current_estimated_millis = last_tc_info
                .packet
                .millis()
                .saturating_add(elapsed_since_last_tc);
        } else {
            // No timecode received yet, so we're at 0
            self.current_estimated_millis = 0;
        }

        self.current_estimated_millis
    }

    pub fn estimated_millis(&self) -> u64 {
        self.current_estimated_millis
    }

    pub fn estimated_timecode(&self) -> TimecodePacket {
        if let Some(last_tc) = self.timecode_history.back() {
            let elapsed_since_last_tc = last_tc.received_at.elapsed().as_millis() as u64;
            let estimated_millis = last_tc
                .packet
                .millis()
                .saturating_add(elapsed_since_last_tc);

            TimecodePacket::from_millis(estimated_millis, last_tc.packet.rate)
        } else {
            TimecodePacket::default()
        }
    }
}
