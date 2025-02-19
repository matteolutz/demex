use egui_probe::EguiProbe;
use serde::{Deserialize, Serialize};

use crate::{
    fixture::{
        handler::{error::FixtureHandlerError, FixtureHandler},
        presets::PresetHandler,
        updatables::UpdatableHandler,
    },
    parser::nodes::{
        action::Action,
        fixture_selector::{FixtureSelector, FixtureSelectorContext},
    },
};

use super::error::DemexInputDeviceError;

#[derive(Debug, Serialize, Deserialize, EguiProbe, Default, Clone)]
pub enum DemexInputButton {
    ExecutorStartAndNext(u32),
    ExecutorStop(u32),
    ExecutorFlash(u32),

    FaderGo(u32),

    SelectivePreset {
        #[egui_probe(skip)]
        fixture_selector: FixtureSelector,
        preset_id: u32,
    },

    FixtureSelector {
        #[egui_probe(skip)]
        fixture_selector: FixtureSelector,
    },

    Macro {
        #[egui_probe(skip)]
        action: Action,
    },

    #[default]
    Unused,
}

impl DemexInputButton {
    pub fn handle_press(
        &self,
        fixture_handler: &mut FixtureHandler,
        preset_handler: &PresetHandler,
        updatable_handler: &mut UpdatableHandler,
        fixture_selector_context: FixtureSelectorContext,
        macro_exec_cue: &mut Vec<Action>,
        global_fixture_selector: &mut Option<FixtureSelector>,
    ) -> Result<(), DemexInputDeviceError> {
        match self {
            Self::ExecutorFlash(executor_id) => updatable_handler
                .start_executor(*executor_id, fixture_handler)
                .map_err(DemexInputDeviceError::UpdatableHandlerError)?,
            Self::ExecutorStartAndNext(executor_id) => updatable_handler
                .start_or_next_executor(*executor_id, fixture_handler, preset_handler)
                .map_err(DemexInputDeviceError::UpdatableHandlerError)?,
            Self::ExecutorStop(executor_id) => updatable_handler
                .stop_executor(*executor_id, fixture_handler)
                .map_err(DemexInputDeviceError::UpdatableHandlerError)?,
            Self::FaderGo(fader_id) => updatable_handler
                .fader_mut(*fader_id)
                .and_then(|f| f.sequence_go(preset_handler))
                .map_err(DemexInputDeviceError::UpdatableHandlerError)?,
            Self::SelectivePreset {
                fixture_selector,
                preset_id,
            } => {
                let preset = preset_handler
                    .get_preset(*preset_id)
                    .map_err(DemexInputDeviceError::PresetHandlerError)?;

                for fixture_id in fixture_selector
                    .get_fixtures(preset_handler, fixture_selector_context)
                    .map_err(DemexInputDeviceError::FixtureSelectorError)?
                {
                    preset
                        .apply(fixture_handler.fixture(fixture_id).ok_or_else(|| {
                            DemexInputDeviceError::FixtureHandlerError(
                                FixtureHandlerError::FixtureNotFound(fixture_id),
                            )
                        })?)
                        .map_err(DemexInputDeviceError::PresetHandlerError)?;
                }
            }
            Self::Macro { action } => {
                macro_exec_cue.push(action.clone());
            }
            Self::FixtureSelector { fixture_selector } => {
                *global_fixture_selector = Some(fixture_selector.clone());
            }
            Self::Unused => {}
        }
        Ok(())
    }

    pub fn handle_release(
        &self,
        fixture_handler: &mut FixtureHandler,
        _preset_handler: &PresetHandler,
        updatable_handler: &mut UpdatableHandler,
    ) -> Result<(), DemexInputDeviceError> {
        match self {
            Self::ExecutorFlash(executor_id) => updatable_handler
                .stop_executor(*executor_id, fixture_handler)
                .map_err(DemexInputDeviceError::UpdatableHandlerError)?,
            _ => {}
        }
        Ok(())
    }
}
