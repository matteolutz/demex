use super::{midi::MidiQuarterTimecodePiece, timecode::packet::TimecodePacket};

#[derive(Debug, Copy, Clone)]
pub enum EncoderValue {
    Absolute(f32),
    RelativeChange(f32),
}

#[derive(Debug, Clone)]
pub enum DemexInputDeviceMessage {
    ButtonPressed(u32),
    ButtonReleased(u32),

    FaderValueChanged(u32, f32),
    FaderTouch(u32),

    FaderValuesChanged(Vec<(u32, f32)>),

    /// These encoders are always automatically mapped to the encoders currently visible in the encoder bar
    GlobalEncoderValueChanged {
        encoder_idx: u32,
        value: EncoderValue,
    },
    GlobalEncoderClick(u32),

    Timecode(TimecodePacket),
    TimecodeQuarterFrame {
        piece: MidiQuarterTimecodePiece,
    },
}
