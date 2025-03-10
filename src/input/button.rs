use std::time;

use egui_probe::EguiProbe;
use serde::{Deserialize, Serialize};

use crate::{
    fixture::{
        handler::{error::FixtureHandlerError, FixtureHandler},
        presets::{preset::FixturePresetId, PresetHandler},
        selection::FixtureSelection,
        timing::TimingHandler,
        updatables::{
            error::UpdatableHandlerError, executor::config::ExecutorConfig, UpdatableHandler,
        },
    },
    parser::nodes::{
        action::Action,
        fixture_selector::{FixtureSelector, FixtureSelectorContext, FixtureSelectorError},
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
        selection: Option<FixtureSelection>,
        preset_id: FixturePresetId,
    },

    FixtureSelector {
        #[egui_probe(skip)]
        fixture_selector: FixtureSelector,
    },

    Macro {
        #[egui_probe(skip)]
        action: Action,
    },

    SpeedMasterTap {
        speed_master_id: u32,
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
        timing_handler: &mut TimingHandler,
        fixture_selector_context: FixtureSelectorContext,
        macro_exec_cue: &mut Vec<Action>,
        global_fixture_selection: &mut Option<FixtureSelection>,
    ) -> Result<(), DemexInputDeviceError> {
        match self {
            Self::ExecutorFlash(executor_id) => updatable_handler
                .start_executor(*executor_id, fixture_handler)
                .map_err(DemexInputDeviceError::UpdatableHandlerError)?,
            Self::ExecutorStartAndNext(executor_id) => {
                let executor = updatable_handler.executor(*executor_id).ok_or(
                    DemexInputDeviceError::UpdatableHandlerError(
                        UpdatableHandlerError::UpdatableNotFound(*executor_id),
                    ),
                )?;

                if matches!(executor.config(), ExecutorConfig::FeatureEffect { .. })
                    && executor.is_started()
                {
                    updatable_handler
                        .stop_executor(*executor_id, fixture_handler)
                        .map_err(DemexInputDeviceError::UpdatableHandlerError)?;
                } else {
                    updatable_handler
                        .start_or_next_executor(*executor_id, fixture_handler, preset_handler)
                        .map_err(DemexInputDeviceError::UpdatableHandlerError)?;
                }
            }
            Self::ExecutorStop(executor_id) => updatable_handler
                .stop_executor(*executor_id, fixture_handler)
                .map_err(DemexInputDeviceError::UpdatableHandlerError)?,
            Self::FaderGo(fader_id) => updatable_handler
                .fader_mut(*fader_id)
                .and_then(|f| f.sequence_go(preset_handler))
                .map_err(DemexInputDeviceError::UpdatableHandlerError)?,
            Self::SelectivePreset {
                selection,
                preset_id,
            } => {
                let preset = preset_handler
                    .get_preset(*preset_id)
                    .map_err(DemexInputDeviceError::PresetHandlerError)?;

                let selection = if let Some(selection) = selection {
                    Some(selection)
                } else {
                    global_fixture_selection.as_ref()
                }
                .ok_or(DemexInputDeviceError::FixtureSelectorError(
                    FixtureSelectorError::NoFixturesMatched,
                ))?;

                for fixture_id in selection.fixtures() {
                    preset
                        .apply(fixture_handler.fixture(*fixture_id).ok_or_else(|| {
                            DemexInputDeviceError::FixtureHandlerError(
                                FixtureHandlerError::FixtureNotFound(*fixture_id),
                            )
                        })?)
                        .map_err(DemexInputDeviceError::PresetHandlerError)?;
                }
            }
            Self::Macro { action } => {
                macro_exec_cue.push(action.clone());
            }
            Self::FixtureSelector { fixture_selector } => {
                *global_fixture_selection = Some(
                    fixture_selector
                        .get_selection(preset_handler, fixture_selector_context)
                        .map_err(DemexInputDeviceError::FixtureSelectorError)?,
                );
            }
            Self::SpeedMasterTap { speed_master_id } => {
                timing_handler
                    .tap_speed_master_value(*speed_master_id, time::Instant::now())
                    .map_err(DemexInputDeviceError::TimingHandlerError)?;
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
            Self::ExecutorStartAndNext(executor_id) => {
                let executor = updatable_handler.executor(*executor_id).ok_or(
                    DemexInputDeviceError::UpdatableHandlerError(
                        UpdatableHandlerError::UpdatableNotFound(*executor_id),
                    ),
                )?;

                if matches!(executor.config(), ExecutorConfig::FeatureEffect { .. })
                    && executor.is_started()
                {}
            }
            Self::ExecutorFlash(executor_id) => updatable_handler
                .stop_executor(*executor_id, fixture_handler)
                .map_err(DemexInputDeviceError::UpdatableHandlerError)?,
            _ => {}
        }
        Ok(())
    }
}
