use std::time;

use serde::{Deserialize, Serialize};

use crate::{
    command::{
        lexer::token::Token,
        parser::nodes::{
            action::{Action, ActionIssuer, queue::ActionQueue},
            fixture_selector::{FixtureSelector, FixtureSelectorContext, FixtureSelectorError},
        },
    },
    event::DemexEvent,
    fixture::handler::FixtureHandler,
    input::{
        DemexInputDeviceUpdateArgs, control::DemexInputDeviceControlTrait,
        error::DemexInputDeviceError, event::DemexInputDeviceButtonUpdate,
    },
    patch::Patch,
    presets::{PresetHandler, preset::FixturePresetId},
    selection::FixtureSelection,
    timing::TimingHandler,
    updatables::UpdatableHandler,
};

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

    #[warn(deprecated)]
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
    ) -> Result<Option<DemexEvent>, DemexInputDeviceError> {
        let event = match self {
            Self::ExecutorFlash { id, stomp } => {
                updatable_handler
                    .start_executor(*id, fixture_handler, preset_handler, 0.0)
                    .map_err(DemexInputDeviceError::UpdatableHandlerError)?;

                if *stomp {
                    updatable_handler.executor_stomp(*id);
                }

                Some(DemexEvent::ExecutorGo(*id))
            }
            Self::ExecutorGo(executor_id) => {
                updatable_handler
                    .executor_go(*executor_id, fixture_handler, preset_handler, 0.0)
                    .map_err(DemexInputDeviceError::UpdatableHandlerError)?;

                Some(DemexEvent::ExecutorGo(*executor_id))
            }
            Self::ExecutorStop(executor_id) => {
                updatable_handler
                    .stop_executor(*executor_id, fixture_handler, preset_handler)
                    .map_err(DemexInputDeviceError::UpdatableHandlerError)?;

                Some(DemexEvent::ExecutorStop(*executor_id))
            }
            Self::SelectivePreset {
                selection,
                preset_id,
            } => {
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

                None
            }
            Self::Macro { action } => {
                action_queue.enqueue_now(action.clone(), ActionIssuer::Macro);
                None
            }
            Self::FixtureSelector { fixture_selector } => {
                *global_fixture_selection = Some(
                    fixture_selector
                        .get_selection(preset_handler, fixture_selector_context)
                        .map_err(DemexInputDeviceError::FixtureSelectorError)?,
                );

                Some(DemexEvent::FixtureSelectionChanged(
                    fixture_selector.clone(),
                ))
            }
            Self::TokenInsert { tokens } => {
                command_input.extend_from_slice(tokens);
                None
            }
            Self::SpeedMasterTap { speed_master_id } => {
                timing_handler
                    .tap_speed_master_value(*speed_master_id, time::Instant::now())
                    .map_err(DemexInputDeviceError::TimingHandlerError)?;
                None
            }
            Self::Unused => None,
        };

        Ok(event)
    }

    pub fn handle_release(
        &self,
        _fixture_handler: &mut FixtureHandler,
        _preset_handler: &PresetHandler,
        updatable_handler: &mut UpdatableHandler,
    ) -> Result<Option<DemexEvent>, DemexInputDeviceError> {
        let event = match self {
            Self::ExecutorGo(executor_id) => {
                let _executor = updatable_handler
                    .executor(*executor_id)
                    .map_err(DemexInputDeviceError::UpdatableHandlerError)?;
                // TODO
                None
            }
            Self::ExecutorFlash { id, stomp } => {
                updatable_handler
                    .executor_cue_out(*id, 0.0)
                    .map_err(DemexInputDeviceError::UpdatableHandlerError)?;

                if *stomp {
                    updatable_handler.executor_unstomp(*id);
                }

                // Some(DemexEvent::ExecutorStop(*id))
                None
            }
            _ => None,
        };

        Ok(event)
    }
}

impl DemexInputDeviceControlTrait<DemexInputDeviceButtonUpdate> for DemexInputButton {
    fn initial_state(
        &self,
        args: DemexInputDeviceUpdateArgs,
    ) -> Result<DemexInputDeviceButtonUpdate, DemexInputDeviceError> {
        match self {
            Self::ExecutorFlash { id, .. } | Self::ExecutorGo(id) | Self::ExecutorStop(id) => {
                let executor = args
                    .updatable_handler
                    .executor(*id)
                    .map_err(DemexInputDeviceError::UpdatableHandlerError)?;

                if executor.is_active() {
                    Ok(DemexInputDeviceButtonUpdate::ButtonActive)
                } else {
                    Ok(DemexInputDeviceButtonUpdate::ButtonInactive)
                }
            }
            _ => Ok(DemexInputDeviceButtonUpdate::default()),
        }
    }

    fn should_update(
        &self,
        _args: DemexInputDeviceUpdateArgs,
        event: &DemexEvent,
    ) -> Result<Option<DemexInputDeviceButtonUpdate>, DemexInputDeviceError> {
        let update = match self {
            Self::ExecutorFlash { id, .. } | Self::ExecutorGo(id) | Self::ExecutorStop(id) => {
                match event {
                    DemexEvent::ExecutorGo(event_id) if event_id == id => {
                        Some(DemexInputDeviceButtonUpdate::ButtonActive)
                    }
                    DemexEvent::ExecutorStop(event_id) if event_id == id => {
                        Some(DemexInputDeviceButtonUpdate::ButtonInactive)
                    }
                    _ => None,
                }
            }
            Self::FixtureSelector { .. } => None,
            Self::SelectivePreset { .. } => None,
            Self::Macro { .. } => None,
            Self::SpeedMasterTap { .. } => None,
            Self::TokenInsert { .. } => None,
            Self::Unused => None,
        };

        Ok(update)
    }
}
