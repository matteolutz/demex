use std::sync::mpsc;

use led::{ApcMiniMk2ButtonLedColor, ApcMiniMk2ButtonLedMode};

use crate::{
    fixture::{
        presets::{preset::FixturePresetTarget, PresetHandler},
        selection::FixtureSelection,
        timing::TimingHandler,
        updatables::{error::UpdatableHandlerError, UpdatableHandler},
    },
    input::{
        button::DemexInputButton, error::DemexInputDeviceError, message::DemexInputDeviceMessage,
        midi::MidiMessage, DemexInputDeviceProfile,
    },
    parser::nodes::fixture_selector::FixtureSelectorContext,
    utils::version::demex_version,
};

use super::DemexInputDeviceProfileType;

mod led;

// **Ressources**
// https://cdn.inmusicbrands.com/akai/attachments/APC%20mini%20mk2%20-%20Communication%20Protocol%20-%20v1.0.pdf

// const APC_MINI_MK_2_NAME: &str = "APC mini mk2 Control";
const APC_MINI_MK_2_NAME: &str = "APC mini mk2";

pub struct ApcMiniMk2InputDeviceProfile {
    rx: mpsc::Receiver<MidiMessage>,
    midi_out: midir::MidiOutputConnection,
    _midi_in: midir::MidiInputConnection<()>,
}

impl std::fmt::Debug for ApcMiniMk2InputDeviceProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ApcMiniMk2InputDeviceProfile")
    }
}

impl ApcMiniMk2InputDeviceProfile {
    pub fn new() -> Result<Self, DemexInputDeviceError> {
        let (tx, rx) = mpsc::channel();

        println!("creating midi output port");
        let midi_out = midir::MidiOutput::new("demex-midi-output")
            .map_err(|err| DemexInputDeviceError::MidirError(err.into()))?;

        println!("finding midi output ports...");
        let out_ports = midi_out.ports();
        let out_port = out_ports
            .iter()
            .find(|p| {
                let port_name = midi_out.port_name(p);
                println!("port name: {:?}", port_name);

                port_name.is_ok_and(|port_name| port_name == APC_MINI_MK_2_NAME)
            })
            .ok_or(DemexInputDeviceError::InputDeviceNotFound(
                APC_MINI_MK_2_NAME.to_owned(),
            ))?;

        let conn_out = midi_out
            .connect(out_port, APC_MINI_MK_2_NAME)
            .map_err(|err| DemexInputDeviceError::MidirError(err.into()))?;

        let midi_in = midir::MidiInput::new("demex-midi-input")
            .map_err(|err| DemexInputDeviceError::MidirError(err.into()))?;

        let in_ports = midi_in.ports();
        let in_port = in_ports
            .iter()
            .find(|p| {
                midi_in
                    .port_name(p)
                    .is_ok_and(|port_name| port_name == APC_MINI_MK_2_NAME)
            })
            .ok_or(DemexInputDeviceError::InputDeviceNotFound(
                APC_MINI_MK_2_NAME.to_owned(),
            ))
            .unwrap();

        let conn_in = midi_in
            .connect(
                in_port,
                APC_MINI_MK_2_NAME,
                move |_, msg, _| {
                    if let Some(midi_msg) = MidiMessage::from_bytes(msg) {
                        tx.send(midi_msg).unwrap();
                    } else {
                        println!("failed to deserialize midi bytes: {:02X?}", msg);
                    }
                },
                (),
            )
            .map_err(|err| DemexInputDeviceError::MidirError(err.into()))?;

        let mut s = Self {
            rx,
            midi_out: conn_out,
            _midi_in: conn_in,
        };

        s.init()?;

        Ok(s)
    }

    pub fn init(&mut self) -> Result<(), DemexInputDeviceError> {
        let (version_major, version_minor, version_patch) = demex_version();

        self.midi_out
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
            .unwrap();

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

        match button_id {
            // RGB buttons
            0..=63 => {
                self.midi_out
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
                self.midi_out
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
                DemexInputButton::ExecutorStartAndNext(id) => {
                    let is_started = updatable_handler
                        .executor(*id)
                        .ok_or(DemexInputDeviceError::UpdatableHandlerError(
                            UpdatableHandlerError::UpdatableNotFound(*id),
                        ))?
                        .is_started();

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
                        .ok_or(DemexInputDeviceError::UpdatableHandlerError(
                            UpdatableHandlerError::UpdatableNotFound(*id),
                        ))?
                        .is_started();

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
                DemexInputButton::ExecutorFlash(id) => {
                    let is_started = updatable_handler
                        .executor(*id)
                        .ok_or(DemexInputDeviceError::UpdatableHandlerError(
                            UpdatableHandlerError::UpdatableNotFound(*id),
                        ))?
                        .is_started();

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
                DemexInputButton::Macro { .. } => {
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
                DemexInputButton::FaderGo(_) => todo!(),
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
            })
            .collect::<Vec<_>>();

        Ok(values)
    }

    fn profile_type(&self) -> super::DemexInputDeviceProfileType {
        DemexInputDeviceProfileType::ApcMiniMk2
    }
}
