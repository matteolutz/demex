use serde::{Deserialize, Serialize};

use crate::{
    command::parser::nodes::{
        action::{
            Action, ActionRunArgs, ValueOrRange, error::ActionRunError, result::ActionRunResult,
        },
        fixture_selector::FixtureSelector,
    },
    input::control::{
        DemexInputDeviceControlAssignment, DemexInputDeviceControlAssignmentDelegate,
        button::{DemexInputButton, DemexInputButtonAssignment},
        fader::{DemexInputFader, DemexInputFaderAssignment},
    },
    presets::preset::FixturePresetId,
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
    pub fn into_buttons(
        &self,
        args: &ActionRunArgs,
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
                        fs.get_selection(args.preset_handler, args.fixture_selector_context)
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
        Ok(ActionRunResult::AssignMultiple(
            self.mode
                .into_buttons(&args)?
                .into_iter()
                .filter_map(|button| DemexInputButtonAssignment::from_control(button, &args).ok())
                .map(|assignment| DemexInputDeviceControlAssignment::Button {
                    device_idx: self.device_idx,
                    button_id: self.button_id,
                    assignment,
                })
                .collect(),
        ))
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AssignFaderArgsMode {
    Executor(u32),
    Grandmaster,
    Groupmaster(u32),
    Speedmaster(u32),
}

impl From<AssignFaderArgsMode> for DemexInputFader {
    fn from(value: AssignFaderArgsMode) -> Self {
        match value {
            AssignFaderArgsMode::Executor(executor_id) => DemexInputFader::Fader { executor_id },
            AssignFaderArgsMode::Grandmaster => DemexInputFader::Grandmaster,
            AssignFaderArgsMode::Groupmaster(group_id) => DemexInputFader::Groupmaster(group_id),
            AssignFaderArgsMode::Speedmaster(speedmaster_id) => DemexInputFader::SpeedMaster {
                speed_master_id: speedmaster_id,
                bpm_min: 50.0,
                bpm_max: 300.0,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignFaderArgs {
    pub mode: AssignFaderArgsMode,
    pub device_idx: usize,
    pub input_fader_id: u32,
}

impl FunctionDelegate for AssignFaderArgs {
    fn run(&self, args: ActionRunArgs) -> Result<ActionRunResult, ActionRunError> {
        let fader = self.mode.into();
        let assignment = DemexInputFaderAssignment::from_control(fader, &args)
            .map_err(ActionRunError::InputDeviceError)?;

        Ok(ActionRunResult::Assign(
            DemexInputDeviceControlAssignment::Fader {
                device_idx: self.device_idx,
                fader_id: self.input_fader_id,
                assignment,
            },
        ))
    }
}
