use std::time;

use serde::{Deserialize, Serialize};

use crate::timecode::TimecodeRate;

#[derive(Debug, Clone)]
pub struct TimedTimecodePacket {
    pub packet: TimecodePacket,
    pub received_at: time::Instant,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TimecodePacket {
    pub rate: TimecodeRate,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub frame: u8,
}

impl PartialOrd for TimecodePacket {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TimecodePacket {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.rate
            .cmp(&other.rate)
            .then_with(|| self.hour.cmp(&other.hour))
            .then_with(|| self.minute.cmp(&other.minute))
            .then_with(|| self.second.cmp(&other.second))
            .then_with(|| self.frame.cmp(&other.frame))
    }
}

impl TimecodePacket {
    pub fn now(self) -> TimedTimecodePacket {
        TimedTimecodePacket {
            packet: self,
            received_at: time::Instant::now(),
        }
    }
}

impl TimecodePacket {
    pub fn millis(&self) -> u64 {
        let seconds = self.hour as u64 * 3600 + self.minute as u64 * 60 + self.second as u64;
        (seconds * 1000) + (self.frame as u64 * 1000 / self.rate.frames_per_second())
    }

    pub fn frame(&self) -> u64 {
        let seconds = self.hour as u64 * 3600 + self.minute as u64 * 60 + self.second as u64;
        seconds * self.rate.frames_per_second() + self.frame as u64
    }

    pub fn from_frame(frames: u64, rate: TimecodeRate) -> Self {
        let total_seconds = frames / rate.frames_per_second();
        let total_minutes = total_seconds / 60;
        let total_hours = total_minutes / 60;

        let hours = total_hours as u8;
        let minutes = (total_minutes % 60) as u8;
        let seconds = (total_seconds % 60) as u8;
        let frames = (frames % rate.frames_per_second()) as u8;

        Self {
            rate,
            hour: hours,
            minute: minutes,
            second: seconds,
            frame: frames,
        }
    }

    pub fn from_millis(millis: u64, rate: TimecodeRate) -> Self {
        let total_seconds = millis / 1000;
        let total_minutes = total_seconds / 60;
        let total_hours = total_minutes / 60;

        let hours = total_hours as u8;
        let minutes = (total_minutes % 60) as u8;
        let seconds = (total_seconds % 60) as u8;
        let frames = (millis % 1000 * rate.frames_per_second() / 1000) as u8;

        Self {
            rate,
            hour: hours,
            minute: minutes,
            second: seconds,
            frame: frames,
        }
    }
}

impl std::fmt::Display for TimecodePacket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:02}:{:02}:{:02}:{:02}",
            self.hour, self.minute, self.second, self.frame
        )
    }
}
