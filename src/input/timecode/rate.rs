use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimecodeRate {
    Frames24,
    Frames25,
    Frames2997,
    Frames30,
}

impl TimecodeRate {
    pub fn frames_per_second(&self) -> u64 {
        match self {
            TimecodeRate::Frames24 => 24,
            TimecodeRate::Frames25 => 25,
            TimecodeRate::Frames2997 => todo!(),
            TimecodeRate::Frames30 => 30,
        }
    }
}

impl From<u8> for TimecodeRate {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Frames24,
            1 => Self::Frames25,
            3 => Self::Frames2997,
            4 => Self::Frames30,
            _ => unreachable!(),
        }
    }
}
