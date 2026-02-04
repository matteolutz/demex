use serde::{Deserialize, Serialize};

use crate::{
    command::parser::nodes::{
        action::{
            Action, ActionRunArgs, ValueOrRange, error::ActionRunError, result::ActionRunResult,
        },
        fixture_selector::FixtureSelector,
    },
    input::control::{
        DemexInputDeviceControlAssignment, button::DemexInputButtonAssignment,
        fader::DemexInputFaderAssignment,
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
    pub fn into_assignments(
        &self,
        args: ActionRunArgs,
    ) -> Result<Vec<DemexInputButtonAssignment>, ActionRunError> {
        match &self {
            AssignButtonArgsMode::ExecutorGo(executor_id) => {
                let is_running = args
                    .updatable_handler
                    .executor(*executor_id)
                    .map_err(ActionRunError::UpdatableHandlerError)?
                    .is_active();

                Ok(vec![DemexInputButtonAssignment::ExecutorGo {
                    executor_id: *executor_id,
                    is_running,
                }])
            }
            AssignButtonArgsMode::ExecutorStop(executor_id) => {
                let is_running = args
                    .updatable_handler
                    .executor(*executor_id)
                    .map_err(ActionRunError::UpdatableHandlerError)?
                    .is_active();

                Ok(vec![DemexInputButtonAssignment::ExecutorStop {
                    executor_id: *executor_id,
                    is_running,
                }])
            }
            AssignButtonArgsMode::ExecutorFlash { id, stomp } => {
                let is_running = args
                    .updatable_handler
                    .executor(*id)
                    .map_err(ActionRunError::UpdatableHandlerError)?
                    .is_active();

                Ok(vec![DemexInputButtonAssignment::ExecutorFlash {
                    executor_id: *id,
                    stomp: *stomp,
                    is_running,
                }])
            }
            AssignButtonArgsMode::FixtureSelector(fixture_selector) => {
                Ok(vec![DemexInputButtonAssignment::FixtureSelector {
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
                    .map(|preset_id| DemexInputButtonAssignment::SelectivePreset {
                        preset_id,
                        selection: selection.clone(),
                    })
                    .collect::<Vec<_>>())
            }
            AssignButtonArgsMode::Macro(action) => Ok(vec![DemexInputButtonAssignment::Macro {
                action: *action.clone(),
            }]),
            AssignButtonArgsMode::SpeedmasterTap(speed_master_id) => {
                // make sure the speed master exists
                args.timing_handler
                    .get_speed_master_value(*speed_master_id)
                    .map_err(ActionRunError::TimingHandlerError)?;

                Ok(vec![DemexInputButtonAssignment::SpeedMasterTap {
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
                .into_assignments(args)?
                .into_iter()
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
        let assignment = match self.mode {
            AssignFaderArgsMode::Executor(executor_id) => {
                // Verify, taht the executor exists
                let executor = args
                    .updatable_handler
                    .executor(executor_id)
                    .map_err(ActionRunError::UpdatableHandlerError)?;
                DemexInputFaderAssignment::executor(executor)
            }
            AssignFaderArgsMode::Grandmaster => {
                let gm_value = args.fixture_handler.grand_master();
                DemexInputFaderAssignment::grandmaster(gm_value)
            }
            AssignFaderArgsMode::Speedmaster(speed_master_id) => {
                let speedmaster = args
                    .timing_handler
                    .get_speed_master_value(speed_master_id)
                    .map_err(ActionRunError::TimingHandlerError)?;
                DemexInputFaderAssignment::speedmaster(speed_master_id, speedmaster, 50.0, 300.0)
            }
        };

        Ok(ActionRunResult::Assign(
            DemexInputDeviceControlAssignment::Fader {
                device_idx: self.device_idx,
                fader_id: self.input_fader_id,
                assignment,
            },
        ))
    }
}
