use serde::{Deserialize, Serialize};

use crate::{
    event::DemexEvent,
    input::{
        DemexInputDeviceUpdateArgs, control::DemexInputDeviceControlTrait,
        error::DemexInputDeviceError, event::DemexInputDeviceFaderUpdate,
    },
    presets::PresetHandler,
    state::fixture_state_handler::FixtureStateHandler,
    timing::TimingHandler,
    updatables::UpdatableHandler,
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
        fixture_handler: &mut FixtureStateHandler,
        preset_handler: &PresetHandler,
        updatable_handler: &mut UpdatableHandler,
        timing_handler: &mut TimingHandler,
    ) -> Result<Option<DemexEvent>, DemexInputDeviceError> {
        let event = match self {
            Self::Fader {
                executor_id: fader_id,
            } => {
                let fader = updatable_handler
                    .executor_mut(*fader_id)
                    .map_err(DemexInputDeviceError::UpdatableHandlerError)?;

                fader.set_value(value, fixture_handler, preset_handler, 0.0);

                Some(DemexEvent::ExecutorFaderValueChanged(*fader_id))
            }
            Self::Groupmaster(id) => {
                let master = updatable_handler
                    .group_master_mut(*id)
                    .map_err(DemexInputDeviceError::UpdatableHandlerError)?;

                *master.value_mut() = value;

                Some(DemexEvent::GroupmasterValueChanged(*id))
            }
            Self::SpeedMaster {
                speed_master_id,
                bpm_min: min_bpm,
                bpm_max: max_bpm,
            } => {
                let speed_master = timing_handler
                    .get_speed_master_value_mut(*speed_master_id)
                    .map_err(DemexInputDeviceError::TimingHandlerError)?;

                let value = min_bpm + (max_bpm - min_bpm) * value;

                speed_master.set_bpm(value);

                Some(DemexEvent::SpeedmasterFaderValueChanged(*speed_master_id))
            }
            Self::Grandmaster => {
                let byte_value = (value * 255.0) as u8;
                *fixture_handler.grand_master_mut() = byte_value;

                Some(DemexEvent::GrandmasterFaderValueChanged)
            }
        };

        Ok(event)
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

impl DemexInputDeviceControlTrait<DemexInputDeviceFaderUpdate> for DemexInputFader {
    fn initial_state(
        &self,
        args: DemexInputDeviceUpdateArgs,
    ) -> Result<DemexInputDeviceFaderUpdate, DemexInputDeviceError> {
        self.value(
            args.fixture_handler,
            args.updatable_handler,
            args.timing_handler,
        )
        .map(DemexInputDeviceFaderUpdate::FaderValueChange)
    }

    fn should_update(
        &self,
        args: DemexInputDeviceUpdateArgs,
        event: &DemexEvent,
    ) -> Result<Option<DemexInputDeviceFaderUpdate>, DemexInputDeviceError> {
        let update = match self {
            Self::Fader { executor_id } => {
                if matches!(event,
                    DemexEvent::ExecutorFaderValueChanged(event_executor_id)
                    | DemexEvent::ExecutorGo(event_executor_id)
                    | DemexEvent::ExecutorStop(event_executor_id) if event_executor_id == executor_id
                ) {
                    let value = args
                        .updatable_handler
                        .executor(*executor_id)
                        .map_err(DemexInputDeviceError::UpdatableHandlerError)?
                        .value();

                    Some(DemexInputDeviceFaderUpdate::FaderValueChange(value))
                } else {
                    None
                }
            }
            Self::Groupmaster(id) => {
                if matches!(event, DemexEvent::GroupmasterValueChanged(event_id) if event_id == id)
                {
                    let value = args
                        .updatable_handler
                        .group_master(*id)
                        .map_err(DemexInputDeviceError::UpdatableHandlerError)?
                        .value();

                    Some(DemexInputDeviceFaderUpdate::FaderValueChange(value))
                } else {
                    None
                }
            }
            Self::Grandmaster => {
                if matches!(event, DemexEvent::GrandmasterFaderValueChanged) {
                    Some(DemexInputDeviceFaderUpdate::FaderValueChange(
                        args.fixture_handler.grand_master() as f32 / 255.0,
                    ))
                } else {
                    None
                }
            }
            Self::SpeedMaster {
                speed_master_id,
                bpm_min,
                bpm_max,
            } => {
                if matches!(event, DemexEvent::SpeedmasterFaderValueChanged(event_speed_master_id) if event_speed_master_id == speed_master_id)
                {
                    let speed_master_value = args
                        .timing_handler
                        .get_speed_master_value(*speed_master_id)
                        .map_err(DemexInputDeviceError::TimingHandlerError)?
                        .bpm();

                    let fader_value = (speed_master_value - bpm_min) / (bpm_max - bpm_min);
                    Some(DemexInputDeviceFaderUpdate::FaderValueChange(
                        fader_value.clamp(0.0, 1.0),
                    ))
                } else {
                    None
                }
            }
        };

        Ok(update)
    }
}
