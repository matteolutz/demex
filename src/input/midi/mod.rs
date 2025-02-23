pub(crate) const NOTE_OFF_OP: u8 = 0x8;
pub(crate) const NOTE_ON_OP: u8 = 0x9;
pub(crate) const CC_OP: u8 = 0xb;

#[derive(Debug)]
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
}

impl MidiMessage {
    pub fn to_bytes(self) -> [u8; 3] {
        match self {
            Self::NoteOn {
                channel,
                note_number,
                key_velocity,
            } => [
                (NOTE_ON_OP << 4) | (channel & 0xF),
                note_number,
                key_velocity,
            ],
            Self::NoteOff {
                channel,
                note_number,
                off_velocity,
            } => [
                (NOTE_OFF_OP << 4) | (channel & 0xF),
                note_number,
                off_velocity,
            ],
            Self::ControlChange {
                channel,
                control_code,
                control_value,
            } => [
                (CC_OP << 4) | (channel & 0xF),
                control_code & 0x7F,
                control_value,
            ],
            Self::AkaiSystemExclusive { .. } => todo!(),
        }
    }

    fn sysex_from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 5 {
            return None;
        }

        let manufacturer_id = bytes[1];
        let device_id = bytes[2];
        let model_id = bytes[3];
        let message_type = bytes[4];

        let data_length: u16 = (bytes[5] as u16) << 8 | bytes[6] as u16;
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
