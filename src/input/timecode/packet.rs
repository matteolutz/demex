use serde::{Deserialize, Serialize};

use super::rate::TimecodeRate;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimecodePacket {
    pub rate: TimecodeRate,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub frame: u8,
}

impl TimecodePacket {
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
}
