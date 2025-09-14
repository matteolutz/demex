use crate::input::{
    error::DemexInputDeviceError,
    message::DemexInputDeviceMessage,
    midi::{device::MidiInOutDevice, device_mode::MidiInOutDeviceMode, MidiMessage},
    DemexInputDeviceProfile,
};

mod encoder;

const GLOBAL_CHANNEL: u8 = 1;

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
                |_| false,
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
        Ok(())
    }
}

impl DemexInputDeviceProfile for BehringerXTouchCompactDeviceProfile {
    fn update_out(
        &mut self,
        _device_config: &crate::input::device::DemexInputDeviceConfig,
        _preset_handler: &crate::fixture::presets::PresetHandler,
        _updatable_handler: &crate::fixture::updatables::UpdatableHandler,
        _timing_handler: &crate::fixture::timing::TimingHandler,
        _global_fixture_selection: &Option<crate::fixture::selection::FixtureSelection>,
    ) -> Result<(), DemexInputDeviceError> {
        todo!()
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
                    key_velocity,
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
                        // Top encoders turn (page A)
                        10..=17 => Some(DemexInputDeviceMessage::GlobalEncoderValueChanged {
                            encoder_idx: (control_code - 10) as u32,
                            value: control_value as f32 / 255.0,
                        }),
                        // Top encoders turn (page B)
                        37..=44 => Some(DemexInputDeviceMessage::GlobalEncoderValueChanged {
                            encoder_idx: control_code as u32 - (37 - 8),
                            value: control_value as f32 / 255.0,
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
