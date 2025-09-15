use crate::{
    fixture::{handler::FixtureHandler, patch::Patch},
    input::{
        encoder::get_global_encoder_value,
        error::DemexInputDeviceError,
        message::DemexInputDeviceMessage,
        midi::{device::MidiInOutDevice, device_mode::MidiInOutDeviceMode, MidiMessage},
        profile::behringer::encoder::BehringerXTouchCompactEncoderMode,
        DemexInputDeviceProfile,
    },
    parser::nodes::fixture_selector::FixtureSelectorContext,
    ui::context::EncoderChannels,
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
}

impl DemexInputDeviceProfile for BehringerXTouchCompactDeviceProfile {
    fn update_out(
        &mut self,
        device_config: &crate::input::device::DemexInputDeviceConfig,
        fixture_handler: &FixtureHandler,
        preset_handler: &crate::fixture::presets::PresetHandler,
        updatable_handler: &crate::fixture::updatables::UpdatableHandler,
        timing_handler: &crate::fixture::timing::TimingHandler,
        global_fixture_selection: &Option<crate::fixture::selection::FixtureSelection>,
        patch: &Patch,
        encoder_channels: Option<&EncoderChannels>,
    ) -> Result<(), DemexInputDeviceError> {
        for (global_encoder_idx, _) in encoder_channels.iter().take(16).enumerate() {
            let encoder_value = get_global_encoder_value(
                global_encoder_idx as u32,
                FixtureSelectorContext::new(global_fixture_selection),
                fixture_handler,
                preset_handler,
                timing_handler,
                encoder_channels,
                patch,
            )
            .unwrap_or_default();

            self.midi.send(MidiMessage::ControlChange {
                channel: GLOBAL_CHANNEL,
                control_code: self.get_encoder_cc(global_encoder_idx as u32)?,
                control_value: (encoder_value * 127.0) as u8,
            })?;
        }

        for (fader_idx, fader) in device_config.faders().iter() {
            let control_value =
                (fader.value(fixture_handler, updatable_handler, timing_handler)? * 127.0) as u8;

            self.midi.send(MidiMessage::ControlChange {
                channel: GLOBAL_CHANNEL,
                control_code: self.get_fader_cc(*fader_idx)?,
                control_value,
            })?;
        }

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
