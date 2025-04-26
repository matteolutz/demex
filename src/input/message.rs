use super::{midi::MidiQuarterTimecodePiece, timecode::packet::TimecodePacket};

pub enum DemexInputDeviceMessage {
    ButtonPressed(u32),
    ButtonReleased(u32),

    FaderValueChanged(u32, f32),
    FaderValuesChanged(Vec<(u32, f32)>),

    Timecode(TimecodePacket),
    TimecodeQuarterFrame { piece: MidiQuarterTimecodePiece },
}
