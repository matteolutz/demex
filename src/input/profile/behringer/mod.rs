use crate::input::{
    error::DemexInputDeviceError,
    event::{
        DemexInputDeviceButtonUpdate, DemexInputDeviceControlUpdate, DemexInputDeviceFaderUpdate,
    },
    message::DemexInputDeviceMessage,
    midi::{device::MidiInOutDevice, device_mode::MidiInOutDeviceMode, MidiMessage},
    profile::behringer::encoder::BehringerXTouchCompactEncoderMode,
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
                |name| name == "X-TOUCH COMPACT",
                MidiInOutDeviceMode::Both,
            ),
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
                control_code: self.get_encoder_cc(global_encoder_idx)?,
                control_value: BehringerXTouchCompactEncoderMode::Single.value(),
            })?;
        }

        Ok(())
    }

    fn get_fader_cc(&self, fader_idx: u32) -> Result<u8, DemexInputDeviceError> {
        match fader_idx {
            0..=8 => Ok(fader_idx as u8 + 1),
            9..=17 => Ok(fader_idx as u8 + (28 - 9)),
            _ => Err(DemexInputDeviceError::FaderNotInProfile),
        }
    }

    fn get_encoder_cc(&self, encoder_idx: u32) -> Result<u8, DemexInputDeviceError> {
        match encoder_idx {
            0..=7 => Ok(encoder_idx as u8 + 10),
            8..=15 => Ok(encoder_idx as u8 + (37 - 8)),
            _ => Err(DemexInputDeviceError::EncoderNotInProfile),
        }
    }

    fn send_fader_value(
        &mut self,
        fader_id: u32,
        fader_value: f32,
    ) -> Result<(), DemexInputDeviceError> {
        self.midi.send(MidiMessage::ControlChange {
            channel: GLOBAL_CHANNEL,
            control_code: self.get_fader_cc(fader_id)?,
            control_value: (fader_value * 127.0) as u8,
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
                DemexInputDeviceControlUpdate::Button { update, .. } => match update {
                    DemexInputDeviceButtonUpdate::ButtonActive => {
                        // TODO
                    }
                    DemexInputDeviceButtonUpdate::ButtonInactive => {
                        // TODO
                    }
                },
            }
        }

        Ok(())
    }

    fn tick(&mut self, _args: DemexInputDeviceUpdateArgs) -> Result<(), DemexInputDeviceError> {
        // TODO: speed master buttons (blinking)

        Ok(())
    }

    fn poll(
        &self,
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
                        10..=17 => Some(DemexInputDeviceMessage::GlobalEncoderValueChanged {
                            encoder_idx: (control_code - 10) as u32,
                            value: control_value as f32 / 127.0,
                        }),
                        // Top encoders turn (page B)
                        37..=44 => Some(DemexInputDeviceMessage::GlobalEncoderValueChanged {
                            encoder_idx: control_code as u32 - (37 - 8),
                            value: control_value as f32 / 127.0,
                        }),
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
