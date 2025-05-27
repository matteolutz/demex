use std::time;

use serde::{Deserialize, Serialize};

use crate::{
    fixture::{
        handler::FixtureHandler,
        patch::Patch,
        presets::{preset::FixturePresetId, PresetHandler},
        selection::FixtureSelection,
        timing::TimingHandler,
        updatables::UpdatableHandler,
    },
    lexer::token::Token,
    parser::nodes::{
        action::{queue::ActionQueue, Action},
        fixture_selector::{FixtureSelector, FixtureSelectorContext, FixtureSelectorError},
    },
};

use super::error::DemexInputDeviceError;

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[cfg_attr(feature = "ui", derive(egui_probe::EguiProbe))]
pub enum DemexInputButton {
    ExecutorGo(u32),
    ExecutorStop(u32),
    ExecutorFlash {
        id: u32,
        stomp: bool,
    },

    SelectivePreset {
        #[cfg_attr(feature = "ui", egui_probe(skip))]
        selection: Option<FixtureSelection>,
        preset_id: FixturePresetId,
    },

    FixtureSelector {
        #[cfg_attr(feature = "ui", egui_probe(skip))]
        fixture_selector: FixtureSelector,
    },

    SpeedMasterTap {
        speed_master_id: u32,
    },

    Macro {
        #[cfg_attr(feature = "ui", egui_probe(skip))]
        action: Action,
    },

    TokenInsert {
        #[cfg_attr(feature = "ui", egui_probe(skip))]
        tokens: Vec<Token>,
    },

    #[default]
    Unused,
}

impl DemexInputButton {
    pub fn handle_press(
        &self,
        fixture_handler: &mut FixtureHandler,
        preset_handler: &mut PresetHandler,
        updatable_handler: &mut UpdatableHandler,
        timing_handler: &mut TimingHandler,
        patch: &Patch,
        fixture_selector_context: FixtureSelectorContext,
        action_queue: &mut ActionQueue,
        global_fixture_selection: &mut Option<FixtureSelection>,
        command_input: &mut Vec<Token>,
    ) -> Result<(), DemexInputDeviceError> {
        match self {
            Self::ExecutorFlash { id, stomp } => {
                updatable_handler
                    .start_fader(*id, fixture_handler, preset_handler, 0.0)
                    .map_err(DemexInputDeviceError::UpdatableHandlerError)?;

                if *stomp {
                    updatable_handler.fader_stomp(*id);
                }
            }
            Self::ExecutorGo(executor_id) => {
                updatable_handler
                    .fader_go(*executor_id, fixture_handler, preset_handler, 0.0)
                    .map_err(DemexInputDeviceError::UpdatableHandlerError)?;
            }
            Self::ExecutorStop(executor_id) => updatable_handler
                .stop_fader(*executor_id, fixture_handler, preset_handler)
                .map_err(DemexInputDeviceError::UpdatableHandlerError)?,
            Self::SelectivePreset {
                selection,
                preset_id,
            } => {
                /*let preset = preset_handler
                .get_preset(*preset_id)
                .map_err(DemexInputDeviceError::PresetHandlerError)?;*/

                let selection = if let Some(selection) = selection {
                    Some(selection)
                } else {
                    global_fixture_selection.as_ref()
                }
                .ok_or(DemexInputDeviceError::FixtureSelectorError(
                    FixtureSelectorError::NoFixturesMatched,
                ))?;

                preset_handler
                    .apply_preset(
                        *preset_id,
                        fixture_handler,
                        patch.fixture_types(),
                        selection.clone(),
                    )
                    .map_err(DemexInputDeviceError::PresetHandlerError)?;
            }
            Self::Macro { action } => {
                action_queue.enqueue_now(action.clone());
            }
            Self::FixtureSelector { fixture_selector } => {
                *global_fixture_selection = Some(
                    fixture_selector
                        .get_selection(preset_handler, fixture_selector_context)
                        .map_err(DemexInputDeviceError::FixtureSelectorError)?,
                );
            }
            Self::TokenInsert { tokens } => {
                command_input.extend_from_slice(tokens);
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
        preset_handler: &PresetHandler,
        updatable_handler: &mut UpdatableHandler,
    ) -> Result<(), DemexInputDeviceError> {
        match self {
            Self::ExecutorGo(executor_id) => {
                let _executor = updatable_handler
                    .fader(*executor_id)
                    .map_err(DemexInputDeviceError::UpdatableHandlerError)?;
                // TODO
            }
            Self::ExecutorFlash { id, stomp } => {
                updatable_handler
                    .stop_fader(*id, fixture_handler, preset_handler)
                    .map_err(DemexInputDeviceError::UpdatableHandlerError)?;

                if *stomp {
                    updatable_handler.fader_unstomp(*id);
                }
            }
            _ => {}
        }
        Ok(())
    }
}
