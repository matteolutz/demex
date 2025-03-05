use device::{DemexInputDevice, DemexInputDeviceConfig};
use error::DemexInputDeviceError;
use message::DemexInputDeviceMessage;
use profile::DemexInputDeviceProfileType;

use crate::{
    fixture::{
        handler::FixtureHandler, presets::PresetHandler, selection::FixtureSelection,
        timing::TimingHandler, updatables::UpdatableHandler,
    },
    lexer::token::Token,
    parser::nodes::{action::Action, fixture_selector::FixtureSelectorContext},
};

pub mod button;
pub mod device;
pub mod error;
pub mod fader;
pub mod message;
pub mod midi;
pub mod profile;

pub trait DemexInputDeviceProfile: std::fmt::Debug {
    fn update_out(
        &mut self,
        device_config: &DemexInputDeviceConfig,
        preset_handler: &PresetHandler,
        updatable_handler: &UpdatableHandler,
        timing_handler: &TimingHandler,
        global_fixture_selection: &Option<FixtureSelection>,
    ) -> Result<(), DemexInputDeviceError>;

    fn poll(&self) -> Result<Vec<DemexInputDeviceMessage>, DemexInputDeviceError>;

    fn profile_type(&self) -> DemexInputDeviceProfileType;

    fn is_enabled(&self) -> bool;
}

#[derive(Default, Debug)]
pub struct DemexInputDeviceHandler {
    devices: Vec<DemexInputDevice>,
}

impl DemexInputDeviceHandler {
    pub fn new(devices: Vec<DemexInputDevice>) -> Self {
        Self { devices }
    }

    pub fn device_mut(
        &mut self,
        idx: usize,
    ) -> Result<&mut DemexInputDevice, DemexInputDeviceError> {
        self.devices
            .get_mut(idx)
            .ok_or(DemexInputDeviceError::InputDeviceIdxNotFound(idx))
    }

    pub fn devices(&self) -> &Vec<DemexInputDevice> {
        &self.devices
    }

    pub fn update(
        &mut self,
        fixture_handler: &mut FixtureHandler,
        preset_handler: &mut PresetHandler,
        updatable_handler: &mut UpdatableHandler,
        timing_handler: &mut TimingHandler,
        fixture_selector_context: FixtureSelectorContext,
        macro_exec_cue: &mut Vec<Action>,
        global_fixture_selection: &mut Option<FixtureSelection>,
        command_input: &mut Vec<Token>,
    ) -> Result<(), DemexInputDeviceError> {
        for (device_idx, device) in self.devices.iter().enumerate() {
            if !device.profile().is_enabled() {
                continue;
            }

            for device_message in device.profile().poll()? {
                match device_message {
                    DemexInputDeviceMessage::ButtonPressed(button_id) => {
                        if let Ok(button) = device
                            .config()
                            .buttons()
                            .get(&button_id)
                            .ok_or(DemexInputDeviceError::ButtonNotFound(button_id))
                        {
                            button.handle_press(
                                fixture_handler,
                                preset_handler,
                                updatable_handler,
                                timing_handler,
                                fixture_selector_context.clone(),
                                macro_exec_cue,
                                global_fixture_selection,
                                command_input,
                            )?;
                        } else {
                            command_input.extend_from_slice(&[Token::FloatingPoint(
                                0.0,
                                (device_idx as u32, button_id),
                            )]);
                        }
                    }
                    DemexInputDeviceMessage::ButtonReleased(button_id) => {
                        let button = device
                            .config()
                            .buttons()
                            .get(&button_id)
                            .ok_or(DemexInputDeviceError::ButtonNotFound(button_id))?;
                        button.handle_release(
                            fixture_handler,
                            preset_handler,
                            updatable_handler,
                        )?;
                    }
                    DemexInputDeviceMessage::FaderValueChanged(fader_id, value) => {
                        let fader = device
                            .config()
                            .faders()
                            .get(&fader_id)
                            .ok_or(DemexInputDeviceError::ButtonNotFound(fader_id))?;
                        fader.handle_change(
                            value,
                            fixture_handler,
                            updatable_handler,
                            timing_handler,
                        )?;
                    }
                    DemexInputDeviceMessage::FaderValuesChanged(fader_values) => {
                        for (fader_id, value) in fader_values {
                            let fader = device
                                .config()
                                .faders()
                                .get(&fader_id)
                                .ok_or(DemexInputDeviceError::ButtonNotFound(fader_id))?;
                            fader.handle_change(
                                value,
                                fixture_handler,
                                updatable_handler,
                                timing_handler,
                            )?;
                        }
                    }
                }
            }
        }

        for device in &mut self.devices {
            if !device.profile().is_enabled() {
                continue;
            }

            device.profile.update_out(
                &device.config,
                preset_handler,
                updatable_handler,
                timing_handler,
                global_fixture_selection,
            )?;
        }

        Ok(())
    }
}
