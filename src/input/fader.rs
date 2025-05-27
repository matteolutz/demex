use serde::{Deserialize, Serialize};

use crate::fixture::{
    handler::FixtureHandler, presets::PresetHandler, timing::TimingHandler,
    updatables::UpdatableHandler,
};

use super::error::DemexInputDeviceError;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ui", derive(egui_probe::EguiProbe))]
pub enum DemexInputFader {
    Fader {
        executor_id: u32,
    },
    SpeedMaster {
        speed_master_id: u32,
        bpm_min: f32,
        bpm_max: f32,
    },
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
        fixture_handler: &mut FixtureHandler,
        preset_handler: &PresetHandler,
        updatable_handler: &mut UpdatableHandler,
        timing_handler: &mut TimingHandler,
    ) -> Result<(), DemexInputDeviceError> {
        match self {
            Self::Fader {
                executor_id: fader_id,
            } => {
                let fader = updatable_handler
                    .executor_mut(*fader_id)
                    .map_err(DemexInputDeviceError::UpdatableHandlerError)?;

                fader.set_value(value, fixture_handler, preset_handler, 0.0);

                Ok(())
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

                Ok(())
            }
        }
    }
}
