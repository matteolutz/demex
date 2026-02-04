use serde::{Deserialize, Serialize};

use crate::{
    command::parser::nodes::action::{Action, ActionIssuer, queue::ActionQueue},
    event::DemexEvent,
    input::{
        DemexInputDeviceUpdateArgs,
        control::{
            DemexInputControlAssignmentResult, DemexInputDeviceControlAssignmentDelegate,
            DemexInputDeviceControlDelegate,
        },
        error::DemexInputDeviceError,
        event::DemexInputDeviceFaderUpdate,
    },
    state::fixture_state_handler::FixtureStateHandler,
    timing::{TimingHandler, speed_master::SpeedMasterValue},
    updatables::{UpdatableHandler, executor::DemexExecutor},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum DemexInputFader {
    Fader {
        executor_id: u32,
    },
    SpeedMaster {
        speed_master_id: u32,
        bpm_min: f32,
        bpm_max: f32,
    },
    Grandmaster,
    Groupmaster(u32),
}

impl Default for DemexInputFader {
    fn default() -> Self {
        Self::Fader { executor_id: 0 }
    }
}

impl DemexInputFader {
    pub fn new(fader_id: u32) -> Self {
        Self::Fader {
            executor_id: fader_id,
        }
    }

    pub fn handle_change(
        &self,
        value: f32,
        action_queue: &mut ActionQueue,
    ) -> Result<(), DemexInputDeviceError> {
        match self {
            Self::Fader {
                executor_id: fader_id,
            } => {
                action_queue.enqueue_now(
                    Action::ExecutorSetFaderValue(*fader_id, value),
                    ActionIssuer::InputDevice,
                );
            }
            Self::Groupmaster(id) => {
                action_queue.enqueue_now(
                    Action::GroupmasterSetFaderValue(*id, value),
                    ActionIssuer::InputDevice,
                );
            }
            Self::SpeedMaster {
                speed_master_id,
                bpm_min: min_bpm,
                bpm_max: max_bpm,
            } => {
                let bpm = min_bpm + (max_bpm - min_bpm) * value;
                action_queue.enqueue_now(
                    Action::SpeedMasterSetBpm(*speed_master_id, bpm),
                    ActionIssuer::InputDevice,
                );
            }
            Self::Grandmaster => {
                let byte_value = (value * 255.0) as u8;
                action_queue.enqueue_now(
                    Action::GrandmasterSetValue(byte_value),
                    ActionIssuer::InputDevice,
                );
            }
        };

        Ok(())
    }

    pub fn value(
        &self,
        fixture_handler: &FixtureStateHandler,
        updatable_handler: &UpdatableHandler,
        timing_handler: &TimingHandler,
    ) -> Result<f32, DemexInputDeviceError> {
        match self {
            Self::Fader {
                executor_id: fader_id,
            } => {
                let executor = updatable_handler
                    .executor(*fader_id)
                    .map_err(DemexInputDeviceError::UpdatableHandlerError)?;

                Ok(executor.value())
            }
            Self::Groupmaster(id) => {
                let master = updatable_handler
                    .group_master(*id)
                    .map_err(DemexInputDeviceError::UpdatableHandlerError)?;

                Ok(master.value())
            }
            Self::SpeedMaster {
                speed_master_id,
                bpm_min: min_bpm,
                bpm_max: max_bpm,
            } => {
                let speed_master = timing_handler
                    .get_speed_master_value(*speed_master_id)
                    .map_err(DemexInputDeviceError::TimingHandlerError)?;

                Ok(((speed_master.bpm() - min_bpm) / (max_bpm - min_bpm)).clamp(0.0, 1.0))
            }
            Self::Grandmaster => {
                let byte_value = fixture_handler.grand_master();
                Ok(byte_value as f32 / 255.0)
            }
        }
    }
}

impl DemexInputDeviceControlDelegate for DemexInputFader {
    type Update = DemexInputDeviceFaderUpdate;

    fn map_event(
        &self,
        _args: DemexInputDeviceUpdateArgs,
        event: &DemexEvent,
    ) -> Result<Option<Self::Update>, DemexInputDeviceError> {
        let update = match self {
            Self::Fader { executor_id } => match event {
                DemexEvent::ExecutorFaderValueChanged {
                    executor_id: event_executor_id,
                    value,
                } if event_executor_id == executor_id => {
                    Some(DemexInputDeviceFaderUpdate::FaderValueChange(*value))
                }
                _ => None,
            },
            Self::Groupmaster(id) => match event {
                DemexEvent::GroupmasterValueChanged {
                    group_master_id,
                    value,
                } if group_master_id == id => {
                    Some(DemexInputDeviceFaderUpdate::FaderValueChange(*value))
                }
                _ => None,
            },
            Self::Grandmaster => match event {
                DemexEvent::GrandmasterFaderValueChanged(value) => {
                    Some(DemexInputDeviceFaderUpdate::FaderValueChange(*value))
                }
                _ => None,
            },
            Self::SpeedMaster {
                speed_master_id,
                bpm_min,
                bpm_max,
            } => match event {
                DemexEvent::SpeedmasterFaderValueChanged {
                    speed_master_id: event_speed_master_id,
                    bpm,
                } if event_speed_master_id == speed_master_id => {
                    let fader_value = (bpm - bpm_min) / (bpm_max - bpm_min);
                    Some(DemexInputDeviceFaderUpdate::FaderValueChange(
                        fader_value.clamp(0.0, 1.0),
                    ))
                }
                _ => None,
            },
        };

        Ok(update)
    }
}

#[derive(Debug, Clone)]
pub struct DemexInputFaderAssignment {
    pub mode: DemexInputFader,
    pub initial_value: Option<f32>,
}

impl DemexInputFaderAssignment {
    pub fn executor(executor: &DemexExecutor) -> Self {
        Self {
            mode: DemexInputFader::Fader {
                executor_id: executor.id(),
            },
            initial_value: Some(executor.value()),
        }
    }

    pub fn grandmaster(value: u8) -> Self {
        Self {
            mode: DemexInputFader::Grandmaster,
            initial_value: Some(value as f32 / 255.0),
        }
    }

    pub fn speedmaster(
        speedmaster_id: u32,
        speedmaster: &SpeedMasterValue,
        bpm_min: f32,
        bpm_max: f32,
    ) -> Self {
        Self {
            mode: DemexInputFader::SpeedMaster {
                speed_master_id: speedmaster_id,
                bpm_min,
                bpm_max,
            },
            initial_value: Some(speedmaster.bpm()),
        }
    }
}

impl DemexInputDeviceControlAssignmentDelegate for DemexInputFaderAssignment {
    type Control = DemexInputFader;

    fn assign(
        self,
    ) -> Result<super::DemexInputControlAssignmentResult<Self::Control>, DemexInputDeviceError>
    {
        let fader_value = match self.mode {
            DemexInputFader::Groupmaster(_)
            | DemexInputFader::Fader { .. }
            | DemexInputFader::Grandmaster => self.initial_value,
            DemexInputFader::SpeedMaster {
                bpm_min, bpm_max, ..
            } => self
                .initial_value
                .map(|bpm| (bpm - bpm_min) / (bpm_max - bpm_min)),
        };

        Ok(DemexInputControlAssignmentResult {
            control: self.mode,
            init_event: fader_value.map(DemexInputDeviceFaderUpdate::FaderValueChange),
        })
    }
}
