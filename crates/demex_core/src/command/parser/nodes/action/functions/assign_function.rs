use serde::{Deserialize, Serialize};

use crate::{
    command::parser::nodes::{
        action::{
            Action, ActionRunArgs, ValueOrRange, error::ActionRunError, result::ActionRunResult,
        },
        fixture_selector::{FixtureSelector, FixtureSelectorContext},
    },
    input::{
        control::{button::DemexInputButton, fader::DemexInputFader},
        error::DemexInputDeviceError,
    },
    presets::{PresetHandler, preset::FixturePresetId},
};

use super::FunctionDelegate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AssignButtonArgsMode {
    ExecutorGo(u32),
    ExecutorStop(u32),
    ExecutorFlash {
        id: u32,
        stomp: bool,
    },
    FixtureSelector(FixtureSelector),
    SelectivePreset {
        preset_id_range: ValueOrRange<FixturePresetId>,
        fixture_selector: Option<FixtureSelector>,
    },
    Macro(Box<Action>),
    SpeedmasterTap(u32),
}

impl AssignButtonArgsMode {
    pub fn check_existing(
        &self,
        ActionRunArgs {
            updatable_handler,
            preset_handler,
            timing_handler,
            ..
        }: &ActionRunArgs,
    ) -> Result<(), ActionRunError> {
        match self {
            Self::ExecutorStop(id) | Self::ExecutorFlash { id, .. } | Self::ExecutorGo(id) => {
                updatable_handler
                    .executor(*id)
                    .map_err(ActionRunError::UpdatableHandlerError)?;
            }
            Self::SelectivePreset {
                preset_id_range: preset_id,
                ..
            } => {
                let (preset_id_from, preset_id_to) = (*preset_id).into();

                preset_handler
                    .get_preset_range(preset_id_from, preset_id_to)
                    .map_err(ActionRunError::PresetHandlerError)?;
            }
            Self::SpeedmasterTap(speed_master_id) => {
                timing_handler
                    .get_speed_master_value(*speed_master_id)
                    .map_err(ActionRunError::TimingHandlerError)?;
            }
            Self::FixtureSelector(_) | Self::Macro(_) => {}
        };

        Ok(())
    }

    pub fn to_buttons(
        &self,
        fixture_selector_context: FixtureSelectorContext,
        preset_handler: &PresetHandler,
    ) -> Result<Vec<DemexInputButton>, ActionRunError> {
        match &self {
            AssignButtonArgsMode::ExecutorGo(executor_id) => {
                Ok(vec![DemexInputButton::ExecutorGo(*executor_id)])
            }
            AssignButtonArgsMode::ExecutorStop(executor_id) => {
                Ok(vec![DemexInputButton::ExecutorStop(*executor_id)])
            }
            AssignButtonArgsMode::ExecutorFlash { id, stomp } => {
                Ok(vec![DemexInputButton::ExecutorFlash {
                    id: *id,
                    stomp: *stomp,
                }])
            }
            AssignButtonArgsMode::FixtureSelector(fixture_selector) => {
                Ok(vec![DemexInputButton::FixtureSelector {
                    fixture_selector: fixture_selector.clone(),
                }])
            }
            AssignButtonArgsMode::SelectivePreset {
                preset_id_range,
                fixture_selector,
            } => {
                let selection = if let Some(fs) = fixture_selector {
                    Some(
                        fs.get_selection(preset_handler, fixture_selector_context)
                            .map_err(ActionRunError::FixtureSelectorError)?,
                    )
                } else {
                    None
                };

                Ok(preset_id_range
                    .try_into_id_list()
                    .map_err(ActionRunError::PresetHandlerError)?
                    .into_iter()
                    .map(|preset_id| DemexInputButton::SelectivePreset {
                        preset_id,
                        selection: selection.clone(),
                    })
                    .collect::<Vec<_>>())
            }
            AssignButtonArgsMode::Macro(action) => Ok(vec![DemexInputButton::Macro {
                action: *action.clone(),
            }]),
            AssignButtonArgsMode::SpeedmasterTap(speed_master_id) => {
                Ok(vec![DemexInputButton::SpeedMasterTap {
                    speed_master_id: *speed_master_id,
                }])
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignButtonArgs {
    pub mode: AssignButtonArgsMode,
    pub device_idx: usize,
    pub button_id: u32,
}

impl FunctionDelegate for AssignButtonArgs {
    fn run(
        &self,
        args: ActionRunArgs,
    ) -> Result<
        crate::command::parser::nodes::action::result::ActionRunResult,
        crate::command::parser::nodes::action::error::ActionRunError,
    > {
        self.mode.check_existing(&args)?;

        let device = args
            .input_device_handler
            .device_mut(self.device_idx)
            .map_err(ActionRunError::InputDeviceError)?;

        if device.config.buttons().get(&self.button_id).is_some() {
            return Err(ActionRunError::InputDeviceError(
                DemexInputDeviceError::ButtonAlreadyAssigned(self.button_id),
            ));
        }

        for (idx, button) in self
            .mode
            .to_buttons(args.fixture_selector_context, args.preset_handler)?
            .into_iter()
            .enumerate()
        {
            device
                .config
                .buttons_mut()
                .insert(self.button_id + idx as u32, button);
        }

        Ok(ActionRunResult::new())
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AssignFaderArgsMode {
    Executor(u32),
    Grandmaster,
    Speedmaster(u32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignFaderArgs {
    pub mode: AssignFaderArgsMode,
    pub device_idx: usize,
    pub input_fader_id: u32,
}

impl FunctionDelegate for AssignFaderArgs {
    fn run(&self, args: ActionRunArgs) -> Result<ActionRunResult, ActionRunError> {
        let device = args
            .input_device_handler
            .device_mut(self.device_idx)
            .map_err(ActionRunError::InputDeviceError)?;

        if device.config.faders().get(&self.input_fader_id).is_some() {
            return Err(ActionRunError::InputDeviceError(
                DemexInputDeviceError::FaderAlreadyAssigned(self.input_fader_id),
            ));
        }

        let assignment = match self.mode {
            AssignFaderArgsMode::Executor(executor_id) => {
                // Verify, taht the executor exists
                let _ = args
                    .updatable_handler
                    .executor(executor_id)
                    .map_err(ActionRunError::UpdatableHandlerError)?;
                DemexInputFader::Fader { executor_id }
            }
            AssignFaderArgsMode::Grandmaster => DemexInputFader::Grandmaster,
            AssignFaderArgsMode::Speedmaster(speed_master_id) => DemexInputFader::SpeedMaster {
                speed_master_id,
                bpm_min: 50.0,
                bpm_max: 300.0,
            },
        };

        device
            .config
            .faders_mut()
            .insert(self.input_fader_id, assignment);

        Ok(ActionRunResult::new())
    }
}
