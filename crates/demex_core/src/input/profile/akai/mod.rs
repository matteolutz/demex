use led::{ApcMiniMk2ButtonLedColor, ApcMiniMk2ButtonLedMode};

use crate::{
    input::{
        DemexInputDeviceProfile, DemexInputDeviceUpdateArgs,
        control::button::DemexInputButton,
        error::DemexInputDeviceError,
        event::{
            DemexInputDeviceButtonUpdate, DemexInputDeviceControlUpdate,
            DemexInputDeviceFaderUpdate,
        },
        message::DemexInputDeviceMessage,
        midi::{MidiMessage, device::MidiInOutDevice, device_mode::MidiInOutDeviceMode},
    },
    utils::version::demex_version,
};

mod led;

// **Ressources**
// https://cdn.inmusicbrands.com/akai/attachments/APC%20mini%20mk2%20-%20Communication%20Protocol%20-%20v1.0.pdf

pub struct ApcMiniMk2InputDeviceProfile {
    #[allow(dead_code)]
    apc_midi_name: String,

    midi: MidiInOutDevice,
}

impl std::fmt::Debug for ApcMiniMk2InputDeviceProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ApcMiniMk2InputDeviceProfile")
            .field("apc_midi_name", &self.apc_midi_name)
            .finish()
    }
}

impl ApcMiniMk2InputDeviceProfile {
    pub fn new(apc_midi_name: String) -> Self {
        let mut s = Self {
            apc_midi_name,
            midi: MidiInOutDevice::new(
                "APC Mini Mk2".to_owned(),
                |name| {
                    name.contains("APC mini mk2")
                        && (cfg!(target_os = "windows") || name.contains("Contr"))
                },
                MidiInOutDeviceMode::Both,
            ),
        };

        if let Err(err) = s.init() {
            log::warn!("Failed to initialize APC Mini Mk2: {}", err);
        }

        s
    }

    pub fn init(&mut self) -> Result<(), DemexInputDeviceError> {
        let midi_out = self
            .midi
            .output_mut()
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
            .midi
            .output_mut()
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

    fn button_color_and_mode(
        &self,
        button: &DemexInputButton,
        is_active: bool,
    ) -> (ApcMiniMk2ButtonLedColor, ApcMiniMk2ButtonLedMode) {
        match button {
            DemexInputButton::ExecutorGo(_) => (
                ApcMiniMk2ButtonLedColor::Green,
                if is_active {
                    ApcMiniMk2ButtonLedMode::Blinking1o8
                } else {
                    ApcMiniMk2ButtonLedMode::IntensFull
                },
            ),
            DemexInputButton::ExecutorFlash { .. } => (
                ApcMiniMk2ButtonLedColor::White,
                if is_active {
                    ApcMiniMk2ButtonLedMode::Blinking1o8
                } else {
                    ApcMiniMk2ButtonLedMode::IntensFull
                },
            ),
            DemexInputButton::ExecutorStop(_) => (
                ApcMiniMk2ButtonLedColor::Red,
                if is_active {
                    ApcMiniMk2ButtonLedMode::Pulsing1o2
                } else {
                    ApcMiniMk2ButtonLedMode::IntensFull
                },
            ),
            // TODO: preset colors
            DemexInputButton::SelectivePreset { .. } => (
                ApcMiniMk2ButtonLedColor::Yellow,
                if is_active {
                    ApcMiniMk2ButtonLedMode::IntensFull
                } else {
                    ApcMiniMk2ButtonLedMode::Intens10
                },
            ),
            DemexInputButton::Macro { .. } => (
                ApcMiniMk2ButtonLedColor::Blue,
                ApcMiniMk2ButtonLedMode::IntensFull,
            ),
            DemexInputButton::FixtureSelector { .. } => (
                ApcMiniMk2ButtonLedColor::Pink,
                if is_active {
                    ApcMiniMk2ButtonLedMode::Blinking1o8
                } else {
                    ApcMiniMk2ButtonLedMode::IntensFull
                },
            ),
            DemexInputButton::SpeedMasterTap { .. } => (
                ApcMiniMk2ButtonLedColor::DarkViolet,
                ApcMiniMk2ButtonLedMode::IntensFull,
            ),
            DemexInputButton::Unused => (
                ApcMiniMk2ButtonLedColor::Off,
                ApcMiniMk2ButtonLedMode::IntensFull,
            ),
        }
    }
}

impl DemexInputDeviceProfile for ApcMiniMk2InputDeviceProfile {
    fn is_enabled(&self) -> bool {
        self.midi.has_input()
    }

    fn handle_events(
        &mut self,
        events: &[crate::input::event::DemexInputDeviceControlUpdate],
    ) -> Result<(), DemexInputDeviceError> {
        for update in events {
            match update {
                DemexInputDeviceControlUpdate::Fader { update, .. } => match update {
                    DemexInputDeviceFaderUpdate::FaderValueChange(_) => {
                        // no motor faders
                    }
                },
                DemexInputDeviceControlUpdate::GlobalEncoder { .. }
                | DemexInputDeviceControlUpdate::Encoder { .. } => {} // we have no encoders
                DemexInputDeviceControlUpdate::Button { id, button, update } => match update {
                    DemexInputDeviceButtonUpdate::ButtonActive => {
                        let (color, mode) = self.button_color_and_mode(button, true);
                        self.set_button_led(*id, mode, color)?;
                    }
                    DemexInputDeviceButtonUpdate::ButtonInactive => {
                        let (color, mode) = self.button_color_and_mode(button, false);
                        self.set_button_led(*id, mode, color)?;
                    }
                },
            }
        }

        Ok(())
    }

    fn tick(&mut self, _args: DemexInputDeviceUpdateArgs) -> Result<(), DemexInputDeviceError> {
        // TODO: speed master buttons (blink)

        Ok(())
    }

    fn poll(
        &mut self,
    ) -> Result<
        Vec<crate::input::message::DemexInputDeviceMessage>,
        crate::input::error::DemexInputDeviceError,
    > {
        let values = self
            .midi
            .input_rx()
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
