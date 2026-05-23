use crate::timecode::TimecodePacket;

pub trait TimecodePacketArtnetExtension {
    fn from_artnet_timecode(timecode: artnet_protocol::Timecode) -> Self;
}

impl TimecodePacketArtnetExtension for TimecodePacket {
    fn from_artnet_timecode(timecode: artnet_protocol::Timecode) -> Self {
        Self {
            rate: timecode.key_type.into(),
            hour: timecode.hours,
            minute: timecode.minutes,
            second: timecode.seconds,
            frame: timecode.frames,
        }
    }
}
