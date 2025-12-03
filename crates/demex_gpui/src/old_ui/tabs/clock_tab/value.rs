use chrono::Timelike;

use crate::input::timecode::packet::TimecodePacket;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClockValue {
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
    pub frame: Option<u32>,
}

impl ClockValue {
    pub fn from_date_time<T: chrono::TimeZone>(date_time: chrono::DateTime<T>) -> Self {
        Self {
            hour: date_time.hour(),
            minute: date_time.minute(),
            second: date_time.second(),
            frame: None,
        }
    }

    pub fn local_time() -> Self {
        Self::from_date_time(chrono::offset::Local::now())
    }

    pub fn utc_time() -> Self {
        Self::from_date_time(chrono::offset::Utc::now())
    }
}

impl std::fmt::Display for ClockValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(frame) = self.frame {
            write!(
                f,
                "{:02}:{:02}:{:02}:{:02}",
                self.hour, self.minute, self.second, frame
            )
        } else {
            write!(f, "{:02}:{:02}:{:02}", self.hour, self.minute, self.second)
        }
    }
}

impl From<TimecodePacket> for ClockValue {
    fn from(packet: TimecodePacket) -> Self {
        Self {
            hour: packet.hour as u32,
            minute: packet.minute as u32,
            second: packet.second as u32,
            frame: Some(packet.frame as u32),
        }
    }
}
