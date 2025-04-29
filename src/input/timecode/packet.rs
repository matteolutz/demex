use serde::{Deserialize, Serialize};

use crate::input::midi::MidiQuarterTimecodePiece;

use super::rate::TimecodeRate;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TimecodePacket {
    pub rate: TimecodeRate,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub frame: u8,
}

impl TimecodePacket {
    pub fn update_from(&mut self, piece: MidiQuarterTimecodePiece) {
        match piece {
            MidiQuarterTimecodePiece::FrameLs(value) => {
                self.frame &= !(0xF);
                self.frame |= value;
            }
            MidiQuarterTimecodePiece::FrameMs(value) => {
                self.frame &= !(0xF << 4);
                self.frame |= value << 4;
            }
            MidiQuarterTimecodePiece::SecondLs(value) => {
                self.second &= !(0xF);
                self.second |= value;
            }
            MidiQuarterTimecodePiece::SecondMs(value) => {
                self.second &= !(0xF << 4);
                self.second |= value << 4;
            }
            MidiQuarterTimecodePiece::MinuteLs(value) => {
                self.minute &= !(0xF);
                self.minute |= value;
            }
            MidiQuarterTimecodePiece::MinuteMs(value) => {
                self.minute &= !(0xF << 4);
                self.minute |= value << 4;
            }
            MidiQuarterTimecodePiece::HourLs(value) => {
                self.hour &= !(0xF);
                self.hour |= value;
            }
            MidiQuarterTimecodePiece::RateHourMs(value) => {
                self.hour &= !(0xF << 4);
                self.hour |= (value & 0x1) << 4;

                let rate = (value & 0x6) >> 1;
                self.rate = TimecodeRate::from(rate);
            }
        }
    }

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
}
