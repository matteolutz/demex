use demex_dmx::timecode::{TimecodePacket, TimecodeRate};

use crate::input::midi::MidiQuarterTimecodePiece;

pub trait TimecodePacketMidiExtension {
    fn update_from(&mut self, piece: MidiQuarterTimecodePiece);
}

impl TimecodePacketMidiExtension for TimecodePacket {
    fn update_from(&mut self, piece: MidiQuarterTimecodePiece) {
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
}
