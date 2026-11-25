use crate::input::{
    DemexInputDeviceProfile, DemexInputDeviceUpdateArgs,
    error::DemexInputDeviceError,
    event::{
        DemexInputDeviceButtonUpdate, DemexInputDeviceControlUpdate, DemexInputDeviceEncoderUpdate,
        DemexInputDeviceFaderUpdate,
    },
    message::{DemexInputDeviceMessage, EncoderValue},
    midi::{
        MidiMessage,
        device::{MidiInOutDevice, MidiInOutIdentifier},
        device_mode::MidiInOutDeviceMode,
    },
    profile::behringer::encoder::BehringerXTouchCompactButtonLedMode,
};

#[allow(unused)]
mod encoder;

// We receive MIDI messages from the Behringer on this channel,
// and we also need to send back values changes (faders, encoders, etc.)
// on this channel.
const GLOBAL_CHANNEL: u8 = 0;

const ENCODER_SENSITIVTY: f32 = 2.0;

fn midi_filter(name: &str) -> bool {
    name == "X-TOUCH COMPACT"
}

// **Ressources**
// https://media.djmania.net/manuales/pdf/Manual_Behringer_X-Touch_Compact.pdf

pub struct BehringerXTouchCompactDeviceProfile {
    midi_id: MidiInOutIdentifier,
    midi: MidiInOutDevice,
}

impl std::fmt::Debug for BehringerXTouchCompactDeviceProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BehringerXTouchCompactDeviceProfile")
            .field("midi_id", &self.midi_id)
            .finish()
    }
}

impl BehringerXTouchCompactDeviceProfile {
    pub fn new(midi_id: MidiInOutIdentifier) -> Self {
        let mut s = Self {
            midi_id: midi_id.clone(),
            midi: MidiInOutDevice::new(
                "Behringer X-Touch Compact".to_owned(),
                midi_filter,
                MidiInOutDeviceMode::Both,
                None,
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
        Ok(())
    }

    fn get_fader_cc(fader_idx: u32) -> Result<u8, DemexInputDeviceError> {
        if fader_idx <= 8 {
            Ok(fader_idx as u8)
        } else {
            Err(DemexInputDeviceError::FaderNotInProfile)
        }
    }

    fn get_relative_encoder_value(midi_value: u8) -> f32 {
        let normalized = if midi_value < 64 {
            // forward turn
            midi_value as f32 / 64.0
        } else if midi_value > 64 {
            // backward turn
            (midi_value - 64) as f32 / -64.0
        } else {
            0.0
        };

        normalized / ENCODER_SENSITIVTY
    }

    fn send_button_state(
        &mut self,
        button_id: u32,
        button_state: BehringerXTouchCompactButtonLedMode,
    ) -> Result<(), DemexInputDeviceError> {
        self.midi.send(MidiMessage::NoteOn {
            channel: GLOBAL_CHANNEL,
            note_number: button_id as u8,
            key_velocity: button_state.velocity(),
        })
    }

    fn send_fader_value(
        &mut self,
        fader_id: u32,
        fader_value: f32,
    ) -> Result<(), DemexInputDeviceError> {
        let msg = MidiMessage::PitchBend {
            channel: Self::get_fader_cc(fader_id)?,
            value: if fader_value == 1.0 {
                16383
            } else {
                (fader_value * 127.0) as u16
            },
        };

        self.midi.send(msg)
    }
}

impl DemexInputDeviceProfile for BehringerXTouchCompactDeviceProfile {
    fn handle_button_assign(&mut self, _button_id: u32) -> Result<(), DemexInputDeviceError> {
        Ok(())
    }

    fn handle_button_unassign(&mut self, button_id: u32) -> Result<(), DemexInputDeviceError> {
        self.send_button_state(button_id, BehringerXTouchCompactButtonLedMode::Off)
    }

    fn handle_fader_assign(&mut self, _fader_id: u32) -> Result<(), DemexInputDeviceError> {
        Ok(())
    }

    fn handle_fader_unassign(&mut self, fader_id: u32) -> Result<(), DemexInputDeviceError> {
        self.send_fader_value(fader_id, 0.0)
    }

    fn num_global_encoders(&self) -> u32 {
        8
    }

    fn handle_events(
        &mut self,
        events: &[DemexInputDeviceControlUpdate],
    ) -> Result<(), DemexInputDeviceError> {
        for event in events {
            log::debug!("BEHRINGER handling event: {:?}", event);

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
                DemexInputDeviceControlUpdate::GlobalEncoder { id: _, update }
                | DemexInputDeviceControlUpdate::Encoder {
                    id: _,
                    encoder: _,
                    update,
                } => match update {
                    DemexInputDeviceEncoderUpdate::EncoderValueChange(_value) => {
                        // TODO: send value to set the encoder ring leds
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
                        32..=39 => Some(DemexInputDeviceMessage::GlobalEncoderClick(
                            note_number as u32 - 32,
                        )),

                        // fader touch
                        104..=112 => Some(DemexInputDeviceMessage::FaderTouch(
                            note_number as u32 - 104,
                        )),

                        _ => Some(DemexInputDeviceMessage::ButtonPressed(note_number as u32)),
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
                        // top encoder click release
                        32..=39 => None,

                        // fader touch release
                        104..=112 => None,

                        _ => Some(DemexInputDeviceMessage::ButtonReleased(note_number as u32)),
                    }
                }
                MidiMessage::PitchBend { channel, value } => {
                    let value = if value == 16383 {
                        1.0
                    } else {
                        value as f32 / 127.0
                    };

                    Some(DemexInputDeviceMessage::FaderValueChanged(
                        channel as u32,
                        value,
                    ))
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
                        // Top encoders turn (page A)
                        16..=23 => {
                            let encoder_idx = control_code as u32 - 16;

                            Some(DemexInputDeviceMessage::GlobalEncoderValueChanged {
                                encoder_idx,
                                value: EncoderValue::RelativeChange(
                                    Self::get_relative_encoder_value(control_value),
                                ),
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
