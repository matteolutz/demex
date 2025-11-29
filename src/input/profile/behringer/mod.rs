use crate::input::{
    control::motorized::{
        DemexInputMotorizedControlState, DemexInputMotorizedControlStateListTrait,
    },
    error::DemexInputDeviceError,
    event::{
        DemexInputDeviceButtonUpdate, DemexInputDeviceControlUpdate, DemexInputDeviceEncoderUpdate,
        DemexInputDeviceFaderUpdate,
    },
    message::DemexInputDeviceMessage,
    midi::{device::MidiInOutDevice, device_mode::MidiInOutDeviceMode, MidiMessage},
    profile::behringer::encoder::{
        BehringerXTouchCompactButtonLedMode, BehringerXTouchCompactEncoderMode,
    },
    DemexInputDeviceProfile, DemexInputDeviceUpdateArgs,
};

mod encoder;

// We receive MIDI messages from the Behringer on this channel,
// and we also need to send back values changes (faders, encoders, etc.)
// on this channel.
const GLOBAL_CHANNEL: u8 = 0;

// On this channel, we only send the configuration messages (given in the documentation)
// i.e. led ring mode, etc.
const GLOBAL_CONFIG_CHANNEL: u8 = 1;

// **Ressources**
// https://media.djmania.net/manuales/pdf/Manual_Behringer_X-Touch_Compact.pdf

pub struct BehringerXTouchCompactDeviceProfile {
    #[allow(dead_code)]
    xtouch_midi_name: String,

    midi: MidiInOutDevice,
    encoder_states: [DemexInputMotorizedControlState; 32],
}

impl std::fmt::Debug for BehringerXTouchCompactDeviceProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BehringerXTouchCompactDeviceProfile")
            .field("xtouch_midi_name", &self.xtouch_midi_name)
            .finish()
    }
}

impl BehringerXTouchCompactDeviceProfile {
    pub fn new(xtouch_midi_name: String) -> Self {
        let mut s = Self {
            xtouch_midi_name,
            midi: MidiInOutDevice::new(
                "Behringer X-Touch Compact".to_owned(),
                |name| name.contains("X-TOUCH COMPACT"),
                MidiInOutDeviceMode::Both,
            ),
            encoder_states: Default::default(),
        };

        if let Err(err) = s.init() {
            log::error!(
                "Failed to initialize Behringer X-Touch Compact device: {}",
                err
            );
        }

        s
    }

    fn init(&mut self) -> Result<(), DemexInputDeviceError> {
        // Setup top encoders
        for global_encoder_idx in 0..=15 {
            self.midi.send(MidiMessage::ControlChange {
                channel: GLOBAL_CONFIG_CHANNEL,
                control_code: Self::get_encoder_cc(global_encoder_idx)?,
                control_value: BehringerXTouchCompactEncoderMode::Single.value(),
            })?;
        }

        Ok(())
    }

    fn get_fader_cc(fader_idx: u32) -> Result<u8, DemexInputDeviceError> {
        match fader_idx {
            0..=8 => Ok(fader_idx as u8 + 1),
            9..=17 => Ok(fader_idx as u8 + (28 - 9)),
            _ => Err(DemexInputDeviceError::FaderNotInProfile),
        }
    }

    fn get_encoder_cc(encoder_idx: u32) -> Result<u8, DemexInputDeviceError> {
        match encoder_idx {
            0..=7 => Ok(encoder_idx as u8 + 10),
            8..=15 => Ok(encoder_idx as u8 + (37 - 8)),
            _ => Err(DemexInputDeviceError::EncoderNotInProfile),
        }
    }

    fn get_button_id_from_note(note_number: u8) -> Result<u32, DemexInputDeviceError> {
        match note_number {
            // Buttons below faders and on the right (page A) - 15 buttons
            40..=54 => Ok(note_number as u32 - 40),

            // Buttons below faders and on the right (page B) - 15 buttons
            95..=109 => Ok(note_number as u32 - (95 - 15)),

            // Buttons top (page A) - 24 buttons
            16..=39 => Ok(note_number as u32 + 14), // - (16 - 30) <=> +14

            // Buttons top (page B) - 24 buttons
            71..=94 => Ok(note_number as u32 - (71 - 54)),
            _ => Err(DemexInputDeviceError::ButtonNotInProfile),
        }
    }

    fn get_button_note_from_id(id: u32) -> Result<u8, DemexInputDeviceError> {
        match id {
            0..=14 => Ok((id + 40) as u8),
            15..=29 => Ok((id + (95 - 15)) as u8),
            30..=53 => Ok((id - 14) as u8),
            54..=77 => Ok((id + (71 - 54)) as u8),
            _ => Err(DemexInputDeviceError::ButtonNotInProfile),
        }
    }

    fn send_button_state(
        &mut self,
        button_id: u32,
        button_state: BehringerXTouchCompactButtonLedMode,
    ) -> Result<(), DemexInputDeviceError> {
        let button_note = Self::get_button_note_from_id(button_id)?;

        self.midi.send(match button_state {
            BehringerXTouchCompactButtonLedMode::On => MidiMessage::NoteOn {
                channel: GLOBAL_CHANNEL,
                note_number: button_note,
                key_velocity: 1,
            },
            BehringerXTouchCompactButtonLedMode::Blink => MidiMessage::NoteOn {
                channel: GLOBAL_CHANNEL,
                note_number: button_note,
                key_velocity: 2,
            },
            BehringerXTouchCompactButtonLedMode::Off => MidiMessage::NoteOff {
                channel: GLOBAL_CHANNEL,
                note_number: button_note,
                off_velocity: 0,
            },
        })
    }

    fn send_fader_value(
        &mut self,
        fader_id: u32,
        fader_value: f32,
    ) -> Result<(), DemexInputDeviceError> {
        self.midi.send(MidiMessage::ControlChange {
            channel: GLOBAL_CHANNEL,
            control_code: Self::get_fader_cc(fader_id)?,
            control_value: (fader_value * 127.0) as u8,
        })
    }

    fn send_encoder_value(
        &mut self,
        encoder_id: u32,
        encoder_value: f32,
    ) -> Result<(), DemexInputDeviceError> {
        self.midi.send(MidiMessage::ControlChange {
            channel: GLOBAL_CHANNEL,
            control_code: Self::get_encoder_cc(encoder_id)?,
            control_value: (encoder_value * 127.0) as u8,
        })
    }
}

impl DemexInputDeviceProfile for BehringerXTouchCompactDeviceProfile {
    fn handle_button_assign(
        &mut self,
        _button_id: u32,
        _button: &crate::input::control::button::DemexInputButton,
    ) -> Result<(), DemexInputDeviceError> {
        Ok(())
    }

    fn handle_button_unassign(
        &mut self,
        _button_id: u32,
        _button: &crate::input::control::button::DemexInputButton,
    ) -> Result<(), DemexInputDeviceError> {
        Ok(())
    }

    fn handle_fader_assign(
        &mut self,
        _fader_id: u32,
        _fader: &crate::input::control::fader::DemexInputFader,
    ) -> Result<(), DemexInputDeviceError> {
        Ok(())
    }

    fn handle_fader_unassign(
        &mut self,
        _fader_id: u32,
        _fader: &crate::input::control::fader::DemexInputFader,
    ) -> Result<(), DemexInputDeviceError> {
        Ok(())
    }

    fn num_global_encoders(&self) -> u32 {
        16
    }

    fn handle_events(
        &mut self,
        _args: DemexInputDeviceUpdateArgs,
        events: &[DemexInputDeviceControlUpdate],
    ) -> Result<(), DemexInputDeviceError> {
        for event in events {
            match event {
                DemexInputDeviceControlUpdate::Fader {
                    id,
                    fader: _,
                    update,
                } => match update {
                    DemexInputDeviceFaderUpdate::FaderValueChange(value) => {
                        self.send_fader_value(*id, *value)?;
                    }
                },
                DemexInputDeviceControlUpdate::GlobalEncoder { id, update }
                | DemexInputDeviceControlUpdate::Encoder {
                    id,
                    encoder: _,
                    update,
                } => match update {
                    DemexInputDeviceEncoderUpdate::EncoderValueChange(value) => {
                        if let Some(encoder_state) = self.encoder_states.get_mut(*id as usize) {
                            encoder_state.update_value(*value);
                        }
                    }
                },
                DemexInputDeviceControlUpdate::Button { update, id, .. } => match update {
                    DemexInputDeviceButtonUpdate::ButtonActive => {
                        self.send_button_state(*id, BehringerXTouchCompactButtonLedMode::Blink)?;
                    }
                    DemexInputDeviceButtonUpdate::ButtonInactive => {
                        self.send_button_state(*id, BehringerXTouchCompactButtonLedMode::On)?;
                    }
                },
            }
        }

        Ok(())
    }

    fn tick(&mut self, _args: DemexInputDeviceUpdateArgs) -> Result<(), DemexInputDeviceError> {
        // TODO: speed master buttons (blinking)

        for (idx, value) in self.encoder_states.send_values() {
            self.send_encoder_value(idx as u32, value)?;
        }

        Ok(())
    }

    fn poll(
        &mut self,
    ) -> Result<Vec<crate::input::message::DemexInputDeviceMessage>, DemexInputDeviceError> {
        let values = self
            .midi
            .input_rx()
            .try_iter()
            .flat_map(|midi_msg| match midi_msg {
                MidiMessage::NoteOn {
                    channel,
                    note_number,
                    key_velocity: _,
                } => {
                    if channel != GLOBAL_CHANNEL {
                        return None;
                    }

                    match note_number {
                        // Top encoders click (page A)
                        0..=7 => Some(DemexInputDeviceMessage::GlobalEncoderClick(
                            note_number as u32,
                        )),
                        // Top encoders click (page B)
                        55..=62 => Some(DemexInputDeviceMessage::GlobalEncoderClick(
                            note_number as u32 - (55 - 8),
                        )),

                        16..=39 | 40..=54 | 71..=94 | 95..=109 => {
                            Some(DemexInputDeviceMessage::ButtonPressed(
                                Self::get_button_id_from_note(note_number).unwrap(),
                            ))
                        }

                        _ => None,
                    }
                }
                MidiMessage::NoteOff {
                    channel,
                    note_number,
                    off_velocity: _,
                } => {
                    if channel != GLOBAL_CHANNEL {
                        return None;
                    }

                    match note_number {
                        16..=39 | 40..=54 | 71..=94 | 95..=109 => {
                            Some(DemexInputDeviceMessage::ButtonReleased(
                                Self::get_button_id_from_note(note_number).unwrap(),
                            ))
                        }

                        _ => None,
                    }
                }
                MidiMessage::ControlChange {
                    channel,
                    control_code,
                    control_value,
                } => {
                    if channel != GLOBAL_CHANNEL {
                        return None;
                    }

                    match control_code {
                        1..=9 => Some(DemexInputDeviceMessage::FaderValueChanged(
                            control_code as u32 - 1,
                            control_value as f32 / 127.0,
                        )),
                        28..=36 => Some(DemexInputDeviceMessage::FaderValueChanged(
                            control_code as u32 - (28 - 9),
                            control_value as f32 / 127.0,
                        )),

                        // Fader touch (page A)
                        101..=109 => Some(DemexInputDeviceMessage::FaderTouch(
                            control_code as u32 - 101,
                        )),

                        // Fader touch (page B)
                        111..=119 => Some(DemexInputDeviceMessage::FaderTouch(
                            control_code as u32 - (111 - 9),
                        )),

                        // Top encoders turn (page A)
                        10..=17 => {
                            let encoder_idx = control_code as u32 - 10;
                            self.encoder_states[encoder_idx as usize].input();

                            Some(DemexInputDeviceMessage::GlobalEncoderValueChanged {
                                encoder_idx,
                                value: control_value as f32 / 127.0,
                            })
                        }
                        // Top encoders turn (page B)
                        37..=44 => {
                            let encoder_idx = control_code as u32 - (37 - 8);
                            self.encoder_states[encoder_idx as usize].input();

                            Some(DemexInputDeviceMessage::GlobalEncoderValueChanged {
                                encoder_idx,
                                value: control_value as f32 / 127.0,
                            })
                        }
                        _ => None,
                    }
                }
                _ => None,
            })
            .collect::<Vec<_>>();

        Ok(values)
    }

    fn is_enabled(&self) -> bool {
        self.midi.has_input()
    }
}

#[cfg(test)]
mod tests {
    use crate::input::profile::behringer::BehringerXTouchCompactDeviceProfile;

    #[test]
    fn test_button_id_notes_roundtrip() {
        let num_buttons = 78;
        for button_id in 0..num_buttons {
            let button_note =
                BehringerXTouchCompactDeviceProfile::get_button_note_from_id(button_id);
            assert!(button_note.is_ok());

            let roundtrip_button_id =
                BehringerXTouchCompactDeviceProfile::get_button_id_from_note(button_note.unwrap());
            assert!(roundtrip_button_id.is_ok());

            assert_eq!(button_id, roundtrip_button_id.unwrap());
        }
    }
}
