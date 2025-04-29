use utils::{get_lower_7_bit, get_upper_7_bit};

pub mod utils;

pub(crate) const NOTE_OFF_OP: u8 = 0x8;
pub(crate) const NOTE_ON_OP: u8 = 0x9;
pub(crate) const CC_OP: u8 = 0xb;

#[derive(Debug, Clone)]
pub enum MidiQuarterTimecodePiece {
    FrameLs(u8),
    FrameMs(u8),
    SecondLs(u8),
    SecondMs(u8),
    MinuteLs(u8),
    MinuteMs(u8),
    HourLs(u8),
    RateHourMs(u8),
}

#[derive(Debug, Clone)]
pub enum MidiMessage {
    NoteOn {
        channel: u8,
        note_number: u8,
        key_velocity: u8,
    },
    NoteOff {
        channel: u8,
        note_number: u8,
        off_velocity: u8,
    },
    ControlChange {
        channel: u8,
        control_code: u8,
        control_value: u8,
    },
    AkaiSystemExclusive {
        manufacturer_id: u8,
        device_id: u8,
        model_id: u8,
        message_type: u8,
        data_length: u16,
        data: Vec<u8>,
    },
    Timecode {
        rate: u8,
        hour: u8,
        minute: u8,
        second: u8,
        frame: u8,
    },
    TimecodeQuarterFrame {
        piece: MidiQuarterTimecodePiece,
    },
}

impl MidiMessage {
    pub fn to_bytes(self) -> Vec<u8> {
        match self {
            Self::NoteOn {
                channel,
                note_number,
                key_velocity,
            } => vec![
                (NOTE_ON_OP << 4) | (channel & 0xF),
                note_number,
                key_velocity,
            ],
            Self::NoteOff {
                channel,
                note_number,
                off_velocity,
            } => vec![
                (NOTE_OFF_OP << 4) | (channel & 0xF),
                note_number,
                off_velocity,
            ],
            Self::ControlChange {
                channel,
                control_code,
                control_value,
            } => vec![
                (CC_OP << 4) | (channel & 0xF),
                control_code & 0x7F,
                control_value,
            ],
            Self::AkaiSystemExclusive {
                manufacturer_id,
                device_id,
                model_id,
                message_type,
                data_length,
                data,
            } => {
                let mut b = vec![
                    0xF0, // SysEx start
                    manufacturer_id,
                    device_id,
                    model_id,
                    message_type,
                    get_upper_7_bit(data_length),
                    get_lower_7_bit(data_length),
                ];

                b.extend(data);

                b.push(0xF7); // SysEx end //

                b
            }
            Self::Timecode {
                rate,
                hour,
                minute,
                second,
                frame,
            } => vec![
                0xF0, // SysEx start
                0x7F, // Manufacturer ID
                0x7F, // Device ID
                0x01, // Model ID
                0x01, // Message Type
                (rate << 5) | hour,
                minute,
                second,
                frame,
                0xF7, // SysEx end //
            ],
            Self::TimecodeQuarterFrame { .. } => todo!(),
        }
    }

    fn sysex_from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 5 {
            return None;
        }

        // not very clean, but it works for now
        // this is a timecode message
        if bytes[1] == 0x7F && bytes[2] == 0x7F && bytes[3] == 0x01 && bytes[4] == 0x01 {
            let rate_and_hour = bytes[5];

            let rate = rate_and_hour >> 5;
            let hour = rate_and_hour & 0x1F;

            let minute = bytes[6];
            let second = bytes[7];
            let frame = bytes[8];

            return Some(Self::Timecode {
                rate,
                hour,
                minute,
                second,
                frame,
            });
        }

        let manufacturer_id = bytes[1];
        let device_id = bytes[2];
        let model_id = bytes[3];
        let message_type = bytes[4];

        let data_length: u16 = ((bytes[5] as u16) << 8) | bytes[6] as u16;
        let data = bytes[7..bytes.len() - 1].to_vec();

        // check if the last byte is the sysex end byte
        if bytes[bytes.len() - 1] != 0xF7 {
            return None;
        }

        Some(Self::AkaiSystemExclusive {
            manufacturer_id,
            device_id,
            model_id,
            message_type,
            data_length,
            data,
        })
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.is_empty() {
            return None;
        }

        // quarter timecode
        if bytes[0] == 0xF1 {
            let piece_type = (bytes[1] & 0xF0) >> 4;
            let piece_value = bytes[1] & 0x0F;
            let piece = match piece_type {
                0 => MidiQuarterTimecodePiece::FrameLs(piece_value),
                1 => MidiQuarterTimecodePiece::FrameMs(piece_value & 0x1),
                2 => MidiQuarterTimecodePiece::SecondLs(piece_value),
                3 => MidiQuarterTimecodePiece::SecondMs(piece_value & 0x3),
                4 => MidiQuarterTimecodePiece::MinuteLs(piece_value),
                5 => MidiQuarterTimecodePiece::MinuteMs(piece_value & 0x3),
                6 => MidiQuarterTimecodePiece::HourLs(piece_value),
                7 => MidiQuarterTimecodePiece::RateHourMs(piece_value & 0x7),
                _ => unreachable!(),
            };

            return Some(Self::TimecodeQuarterFrame { piece });
        }

        if bytes.len() < 3 {
            return None;
        }

        if bytes[0] == 0xF0 {
            return Self::sysex_from_bytes(bytes);
        }

        let message_type = (bytes[0] & 0xF0) >> 4;
        let channel = bytes[0] & 0xF;

        match message_type {
            NOTE_OFF_OP => Some(Self::NoteOff {
                channel,
                note_number: bytes[1],
                off_velocity: bytes[2],
            }),
            NOTE_ON_OP => {
                let key_velocity = bytes[2];
                if key_velocity == 0 {
                    Some(Self::NoteOff {
                        channel,
                        note_number: bytes[1],
                        off_velocity: 64,
                    })
                } else {
                    Some(Self::NoteOn {
                        channel,
                        note_number: bytes[1],
                        key_velocity,
                    })
                }
            }
            CC_OP => Some(Self::ControlChange {
                channel,
                control_code: bytes[1] & 0x7F,
                control_value: bytes[2],
            }),
            _ => None,
        }
    }
}
