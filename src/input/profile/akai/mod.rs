use std::sync::mpsc;

use led::{ApcMiniMk2ButtonLedColor, ApcMiniMk2ButtonLedMode};

use crate::{
    fixture::{
        presets::{preset::FixturePresetTarget, PresetHandler},
        selection::FixtureSelection,
        timing::TimingHandler,
        updatables::UpdatableHandler,
    },
    input::{
        button::DemexInputButton, error::DemexInputDeviceError, message::DemexInputDeviceMessage,
        midi::MidiMessage, DemexInputDeviceProfile,
    },
    parser::nodes::fixture_selector::FixtureSelectorContext,
    utils::version::demex_version,
};

mod led;

// **Ressources**
// https://cdn.inmusicbrands.com/akai/attachments/APC%20mini%20mk2%20-%20Communication%20Protocol%20-%20v1.0.pdf

pub struct ApcMiniMk2InputDeviceProfile {
    #[allow(dead_code)]
    apc_midi_name: String,

    rx: mpsc::Receiver<MidiMessage>,
    midi_out: Option<midir::MidiOutputConnection>,
    _midi_in: Option<midir::MidiInputConnection<()>>,
}

impl std::fmt::Debug for ApcMiniMk2InputDeviceProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ApcMiniMk2InputDeviceProfile")
    }
}

impl ApcMiniMk2InputDeviceProfile {
    fn get_conn_out(
        apc_midi_name: &str,
    ) -> Result<midir::MidiOutputConnection, DemexInputDeviceError> {
        let midi_out = midir::MidiOutput::new("demex-midi-output")
            .map_err(|err| DemexInputDeviceError::MidirError(err.into()))?;

        let out_ports = midi_out.ports();
        let out_port = out_ports
            .iter()
            .inspect(|p| log::debug!("Found MIDI out port: {:?}", midi_out.port_name(p)))
            .find(|p| {
                midi_out
                    .port_name(p)
                    .is_ok_and(|port_name| port_name == apc_midi_name)
            })
            .ok_or(DemexInputDeviceError::InputDeviceNotFound(
                apc_midi_name.to_owned(),
            ))?;

        midi_out
            .connect(out_port, apc_midi_name)
            .map_err(|err| DemexInputDeviceError::MidirError(err.into()))
    }

    fn get_conn_in(
        apc_midi_name: &str,
        tx: mpsc::Sender<MidiMessage>,
    ) -> Result<midir::MidiInputConnection<()>, DemexInputDeviceError> {
        let midi_in = midir::MidiInput::new("demex-midi-input")
            .map_err(|err| DemexInputDeviceError::MidirError(err.into()))?;

        let in_ports = midi_in.ports();
        let in_port = in_ports
            .iter()
            .inspect(|p| log::debug!("Found MIDI in port: {:?}", midi_in.port_name(p)))
            .find(|p| {
                midi_in
                    .port_name(p)
                    .is_ok_and(|port_name| port_name == apc_midi_name)
            })
            .ok_or(DemexInputDeviceError::InputDeviceNotFound(
                apc_midi_name.to_owned(),
            ))?;

        midi_in
            .connect(
                in_port,
                apc_midi_name,
                move |_, msg, _| {
                    if let Some(midi_msg) = MidiMessage::from_bytes(msg) {
                        tx.send(midi_msg).unwrap();
                    } else {
                        log::debug!("failed to deserialize midi bytes: {:02X?}", msg);
                    }
                },
                (),
            )
            .map_err(|err| DemexInputDeviceError::MidirError(err.into()))
    }

    pub fn new(apc_midi_name: String) -> Self {
        let (tx, rx) = mpsc::channel();

        let conn_out = Self::get_conn_out(&apc_midi_name);
        let conn_in = Self::get_conn_in(&apc_midi_name, tx);

        let mut s = Self {
            apc_midi_name,
            rx,
            midi_out: conn_out
                .inspect_err(|err| {
                    log::warn!(
                        "Failed to establish APC Mini Mk2 MIDI out connection: {}",
                        err
                    )
                })
                .ok(),
            _midi_in: conn_in
                .inspect_err(|err| {
                    log::warn!(
                        "Failed to establish APC Mini Mk2 MIDI in connection: {}",
                        err
                    )
                })
                .ok(),
        };

        if let Err(err) = s.init() {
            log::warn!("Failed to initialize APC Mini Mk2: {}", err);
        }

        s
    }

    pub fn init(&mut self) -> Result<(), DemexInputDeviceError> {
        let midi_out = self
            .midi_out
            .as_mut()
            .ok_or(DemexInputDeviceError::OperationNotSupported)?;

        let (version_major, version_minor, version_patch) = demex_version();

        midi_out
            .send(&[
                0xF0,          // sysex start
                0x47,          // manufacturer id
                0x7F,          // device id
                0x4F,          // mode id
                0x60,          // message type
                0x0,           // hi-bytes to follow
                0x04,          // lo-bytes to follow
                0x0,           // application id
                version_major, // demex major version
                version_minor, // demex minor version
                version_patch, // demex patch version
                0xF7,          // end of sysex
            ])
            .map_err(|err| DemexInputDeviceError::MidirError(err.into()))?;

        for i in (0..=63).chain(100..=107).chain(200..=207).chain(300..=300) {
            self.set_button_led(
                i,
                ApcMiniMk2ButtonLedMode::IntensFull,
                ApcMiniMk2ButtonLedColor::Off,
            )?;
        }

        Ok(())
    }

    pub fn get_button_note_number(&self, idx: u32) -> Option<u8> {
        match idx {
            0..=63 => {
                let row = 7 - (idx / 8);
                let col = idx % 8;
                Some((row * 8 + col) as u8)
            }
            100..=107 => Some((idx - 100) as u8 + 0x64),
            200..=207 => Some((idx - 200) as u8 + 0x70),
            300 => Some(0x7A),
            _ => None,
        }
    }

    pub fn get_button_idx(&self, _channel: u8, note_number: u8) -> Option<u32> {
        match note_number {
            0x00..=0x3F => {
                let row = 7 - (note_number as u32 / 8);
                let col = note_number as u32 % 8;
                Some(row * 8 + col)
            }
            0x64..=0x6B => Some(note_number as u32 - 0x64 + 100),
            0x70..=0x77 => Some(note_number as u32 - 0x70 + 200),
            0x7A => Some(300),
            _ => None,
        }
    }

    pub fn get_fader_idx(&self, channel: u8, control_code: u8) -> Option<u32> {
        if channel != 0 {
            return None;
        }

        match control_code {
            0x30..=0x38 => Some(control_code as u32 - 0x30),
            _ => None,
        }
    }

    pub fn off_button_led(&mut self, button_id: u32) -> Result<(), DemexInputDeviceError> {
        self.set_button_led(
            button_id,
            ApcMiniMk2ButtonLedMode::IntensFull,
            ApcMiniMk2ButtonLedColor::Off,
        )
    }

    pub fn set_button_led(
        &mut self,
        button_id: u32,
        mode: ApcMiniMk2ButtonLedMode,
        color: ApcMiniMk2ButtonLedColor,
    ) -> Result<(), DemexInputDeviceError> {
        let note_number = self
            .get_button_note_number(button_id)
            .ok_or(DemexInputDeviceError::ButtonNotFound(button_id))?;

        let midi_out = self
            .midi_out
            .as_mut()
            .ok_or(DemexInputDeviceError::OperationNotSupported)?;

        match button_id {
            // RGB buttons
            0..=63 => {
                midi_out
                    .send(
                        &MidiMessage::NoteOn {
                            channel: mode.value(),
                            note_number,
                            key_velocity: color.value(),
                        }
                        .to_bytes(),
                    )
                    .map_err(|err| DemexInputDeviceError::MidirError(err.into()))?;
            }
            // static buttons
            _ => {
                midi_out
                    .send(
                        &MidiMessage::NoteOn {
                            channel: 0,
                            note_number,
                            key_velocity: if color == ApcMiniMk2ButtonLedColor::Off {
                                0
                            } else if mode.is_static() {
                                1
                            } else {
                                2
                            },
                        }
                        .to_bytes(),
                    )
                    .map_err(|err| DemexInputDeviceError::MidirError(err.into()))?;
            }
        }

        Ok(())
    }
}

impl DemexInputDeviceProfile for ApcMiniMk2InputDeviceProfile {
    fn is_enabled(&self) -> bool {
        self.midi_out.is_some()
    }

    fn update_out(
        &mut self,
        device_config: &crate::input::device::DemexInputDeviceConfig,
        preset_handler: &PresetHandler,
        updatable_handler: &UpdatableHandler,
        timing_handler: &TimingHandler,
        global_fixture_selection: &Option<FixtureSelection>,
    ) -> Result<(), DemexInputDeviceError> {
        for (button_id, button) in device_config.buttons() {
            match button {
                DemexInputButton::ExecutorGo(id) => {
                    let is_started = updatable_handler
                        .executor(*id)
                        .map_err(DemexInputDeviceError::UpdatableHandlerError)?
                        .is_active();

                    self.set_button_led(
                        *button_id,
                        if !is_started {
                            ApcMiniMk2ButtonLedMode::IntensFull
                        } else {
                            ApcMiniMk2ButtonLedMode::Blinking1o8
                        },
                        ApcMiniMk2ButtonLedColor::Green,
                    )?;
                }
                DemexInputButton::ExecutorStop(id) => {
                    let is_started = updatable_handler
                        .executor(*id)
                        .map_err(DemexInputDeviceError::UpdatableHandlerError)?
                        .is_active();

                    self.set_button_led(
                        *button_id,
                        if !is_started {
                            ApcMiniMk2ButtonLedMode::IntensFull
                        } else {
                            ApcMiniMk2ButtonLedMode::Pulsing1o2
                        },
                        ApcMiniMk2ButtonLedColor::Red,
                    )?;
                }
                DemexInputButton::ExecutorFlash { id, .. } => {
                    let is_started = updatable_handler
                        .executor(*id)
                        .map_err(DemexInputDeviceError::UpdatableHandlerError)?
                        .is_active();

                    self.set_button_led(
                        *button_id,
                        if !is_started {
                            ApcMiniMk2ButtonLedMode::IntensFull
                        } else {
                            ApcMiniMk2ButtonLedMode::Blinking1o8
                        },
                        ApcMiniMk2ButtonLedColor::White,
                    )?;
                }
                DemexInputButton::SelectivePreset {
                    selection,
                    preset_id,
                } => {
                    let preset = preset_handler.get_preset(*preset_id).ok();

                    let target_mode = preset
                        .map(|preset| {
                            preset.get_target(
                                global_fixture_selection
                                    .as_ref()
                                    .map(|selection| selection.fixtures())
                                    .unwrap_or(&[]),
                            )
                        })
                        .unwrap_or(FixturePresetTarget::None);

                    let display_color = preset.and_then(|p| p.display_color());

                    self.set_button_led(
                        *button_id,
                        if selection.is_some() || target_mode == FixturePresetTarget::AllSelected {
                            ApcMiniMk2ButtonLedMode::IntensFull
                        } else {
                            ApcMiniMk2ButtonLedMode::Intens10
                        },
                        display_color
                            .and_then(ApcMiniMk2ButtonLedColor::try_from_color)
                            .unwrap_or_else(|| {
                                if selection.is_some() {
                                    ApcMiniMk2ButtonLedColor::Orange
                                } else {
                                    ApcMiniMk2ButtonLedColor::Yellow
                                }
                            }),
                    )?;
                }
                DemexInputButton::Macro { .. } | DemexInputButton::TokenInsert { .. } => {
                    self.set_button_led(
                        *button_id,
                        ApcMiniMk2ButtonLedMode::IntensFull,
                        ApcMiniMk2ButtonLedColor::Blue,
                    )?;
                }
                DemexInputButton::FixtureSelector { fixture_selector } => {
                    let is_selected = global_fixture_selection.as_ref().is_some_and(|selection| {
                        selection.equals_selector(
                            fixture_selector,
                            preset_handler,
                            FixtureSelectorContext::new(global_fixture_selection),
                        )
                    });

                    self.set_button_led(
                        *button_id,
                        if !is_selected {
                            ApcMiniMk2ButtonLedMode::IntensFull
                        } else {
                            ApcMiniMk2ButtonLedMode::Blinking1o8
                        },
                        ApcMiniMk2ButtonLedColor::Pink,
                    )?;
                }
                DemexInputButton::SpeedMasterTap { speed_master_id } => {
                    let speed_master_value = timing_handler
                        .get_speed_master_value(*speed_master_id)
                        .map_err(DemexInputDeviceError::TimingHandlerError)?;

                    self.set_button_led(
                        *button_id,
                        ApcMiniMk2ButtonLedMode::IntensFull,
                        if speed_master_value.interval().is_none() || speed_master_value.on_beat() {
                            ApcMiniMk2ButtonLedColor::DarkViolet
                        } else {
                            ApcMiniMk2ButtonLedColor::Off
                        },
                    )?;
                }
                DemexInputButton::Unused => {}
            }
        }

        Ok(())
    }

    fn poll(
        &self,
    ) -> Result<
        Vec<crate::input::message::DemexInputDeviceMessage>,
        crate::input::error::DemexInputDeviceError,
    > {
        let values = self
            .rx
            .try_iter()
            .flat_map(|midi_msg| match midi_msg {
                MidiMessage::NoteOn {
                    channel,
                    note_number,
                    ..
                } => self
                    .get_button_idx(channel, note_number)
                    .map(DemexInputDeviceMessage::ButtonPressed),
                MidiMessage::NoteOff {
                    channel,
                    note_number,
                    ..
                } => self
                    .get_button_idx(channel, note_number)
                    .map(DemexInputDeviceMessage::ButtonReleased),
                MidiMessage::ControlChange {
                    channel,
                    control_code,
                    control_value,
                } => self.get_fader_idx(channel, control_code).map(|idx| {
                    DemexInputDeviceMessage::FaderValueChanged(idx, control_value as f32 / 127.0)
                }),
                MidiMessage::AkaiSystemExclusive {
                    message_type, data, ..
                } => {
                    // answer to init message, contains current fader values
                    if message_type == 0x61 && data.len() == 9 {
                        let fader_values = data
                            .iter()
                            .enumerate()
                            .map(|(idx, &v)| (idx as u32, v as f32 / 127.0))
                            .collect::<Vec<_>>();

                        Some(DemexInputDeviceMessage::FaderValuesChanged(fader_values))
                    } else {
                        None
                    }
                }
                _ => None,
            })
            .collect::<Vec<_>>();

        Ok(values)
    }
}
