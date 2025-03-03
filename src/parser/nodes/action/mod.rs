use functions::{
    delete_function::DeleteArgs,
    record_function::{RecordGroupArgs, RecordPresetArgs, RecordSequenceCueArgs},
    rename_function::RenameObjectArgs,
    set_function::{SetFeatureValueArgs, SetFixturePresetArgs},
    ActionFunction,
};
use serde::{Deserialize, Serialize};

use crate::{
    fixture::{
        handler::FixtureHandler,
        presets::{preset::FixturePresetId, PresetHandler},
        updatables::{
            error::UpdatableHandlerError, fader::config::DemexFaderConfig, UpdatableHandler,
        },
    },
    input::{
        button::DemexInputButton, error::DemexInputDeviceError, fader::DemexInputFader,
        DemexInputDeviceHandler,
    },
    ui::{constants::INFO_TEXT, window::edit::DemexEditWindow},
};

use self::{error::ActionRunError, result::ActionRunResult};

use super::{
    fixture_selector::{FixtureSelector, FixtureSelectorContext, FixtureSelectorError},
    object::{HomeableObject, Object, ObjectTrait},
};

pub mod error;
pub mod functions;
pub mod result;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum ValueOrRange<T> {
    Single(T),
    Thru(T, T),
}

impl<T: Copy> From<ValueOrRange<T>> for (T, T) {
    fn from(value: ValueOrRange<T>) -> Self {
        match value {
            ValueOrRange::Single(single) => (single, single),
            ValueOrRange::Thru(from, to) => (from, to),
        }
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum ExecutorAssignmentModeActionData {
    StartAndNext,
    Stop,
    Flash,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum UpdateModeActionData {
    Merge,
    Override,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ConfigTypeActionData {
    Output,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum FaderCreationConfigActionData {
    Submaster(FixtureSelector),
    Sequence(u32, FixtureSelector),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ExecutorCreationModeActionData {
    Sequence(u32),
    Effect,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum Action {
    SetFeatureValue(SetFeatureValueArgs),
    SetFixturePreset(SetFixturePresetArgs),

    Home(HomeableObject),
    HomeAll,

    RecordPreset(RecordPresetArgs),
    RecordGroup2(RecordGroupArgs),
    RecordSequenceCue(RecordSequenceCueArgs),
    RecordMacro(Box<Action>, u32),

    Rename(RenameObjectArgs),

    CreateSequence(Option<u32>, Option<String>),
    CreateExecutor(
        Option<u32>,
        ExecutorCreationModeActionData,
        FixtureSelector,
        Option<String>,
    ),
    CreateMacro(Option<u32>, Box<Action>, Option<String>),
    CreateFader(Option<u32>, FaderCreationConfigActionData, Option<String>),

    UpdatePreset(FixturePresetId, FixtureSelector, UpdateModeActionData),

    Delete(DeleteArgs),

    Edit(Object),

    AssignExecutorToInput {
        executor_id: u32,
        mode: ExecutorAssignmentModeActionData,
        device_idx: usize,
        button_id: u32,
    },
    AssignFaderToInput {
        fader_id: u32,
        device_idx: usize,
        input_fader_id: u32,
    },
    AssignFaderGoToInput {
        fader_id: u32,
        device_idx: usize,
        button_id: u32,
    },
    AssignSelectivePresetToInput {
        preset_id: ValueOrRange<FixturePresetId>,
        fixture_selector: Option<FixtureSelector>,
        device_idx: usize,
        button_id: u32,
    },
    AssignFixtureSelectorToInput {
        fixture_selector: FixtureSelector,
        device_idx: usize,
        button_id: u32,
    },
    AssignMacroToInput {
        action: Box<Action>,
        device_idx: usize,
        button_id: u32,
    },

    UnassignInputButton {
        device_idx: usize,
        button_id: u32,
    },
    UnassignInputFader {
        device_idx: usize,
        fader_id: u32,
    },

    FixtureSelector(FixtureSelector),
    ClearAll,
    Save,
    Test(String),

    Config(ConfigTypeActionData),

    Nuzul,
    Sueud,

    #[default]
    MatteoLutz,
}

impl Action {
    pub fn run(
        &self,
        fixture_handler: &mut FixtureHandler,
        preset_handler: &mut PresetHandler,
        fixture_selector_context: FixtureSelectorContext,
        updatable_handler: &mut UpdatableHandler,
        input_device_handler: &mut DemexInputDeviceHandler,
    ) -> Result<ActionRunResult, ActionRunError> {
        match self {
            Self::SetFeatureValue(args) => args.run(
                fixture_handler,
                preset_handler,
                fixture_selector_context,
                updatable_handler,
                input_device_handler,
            ),
            Self::SetFixturePreset(args) => args.run(
                fixture_handler,
                preset_handler,
                fixture_selector_context,
                updatable_handler,
                input_device_handler,
            ),

            Self::Home(homeable_object) => homeable_object.run_home(
                preset_handler,
                fixture_handler,
                updatable_handler,
                fixture_selector_context,
            ),

            Self::HomeAll => self.run_home_all(fixture_handler),

            Self::RecordPreset(args) => args.run(
                fixture_handler,
                preset_handler,
                fixture_selector_context,
                updatable_handler,
                input_device_handler,
            ),

            Self::RecordGroup2(args) => args.run(
                fixture_handler,
                preset_handler,
                fixture_selector_context,
                updatable_handler,
                input_device_handler,
            ),

            Self::RecordSequenceCue(args) => args.run(
                fixture_handler,
                preset_handler,
                fixture_selector_context,
                updatable_handler,
                input_device_handler,
            ),

            Self::Rename(args) => args.run(
                fixture_handler,
                preset_handler,
                fixture_selector_context,
                updatable_handler,
                input_device_handler,
            ),

            Self::CreateSequence(id, name) => self.run_create_sequence(preset_handler, *id, name),
            Self::CreateExecutor(id, mode, fixture_selector, name) => self.run_create_executor(
                updatable_handler,
                preset_handler,
                *id,
                name,
                mode,
                fixture_selector,
                fixture_selector_context,
            ),
            Self::CreateMacro(id, action, name) => {
                self.run_create_macro(*id, action, name, preset_handler)
            }
            Self::CreateFader(id, config, name) => self.run_create_fader(
                *id,
                config,
                name,
                updatable_handler,
                preset_handler,
                fixture_selector_context,
            ),

            Self::UpdatePreset(preset_id, fixture_selector, update_mode) => self.run_update_preset(
                preset_handler,
                fixture_handler,
                fixture_selector,
                fixture_selector_context,
                *preset_id,
                update_mode,
            ),

            Self::Delete(args) => args.run(
                fixture_handler,
                preset_handler,
                fixture_selector_context,
                updatable_handler,
                input_device_handler,
            ),

            Self::Edit(object) => object
                .clone()
                .edit_window()
                .ok_or(ActionRunError::ActionNotImplementedForObject(
                    "Edit".to_owned(),
                    object.clone(),
                ))
                .map(ActionRunResult::EditWindow),

            Self::ClearAll => Ok(ActionRunResult::new()),
            Self::FixtureSelector(fixture_selector) => self.run_fixture_selector(
                fixture_selector,
                fixture_selector_context,
                preset_handler,
                fixture_handler,
            ),
            Self::Test(_) => Ok(ActionRunResult::new()),
            Self::Save => Ok(ActionRunResult::new()),

            Self::Nuzul => Ok(ActionRunResult::Info("Going down...".to_owned())),
            Self::Sueud => Ok(ActionRunResult::Info("Going up...".to_owned())),

            Self::Config(config_type) => Ok(ActionRunResult::EditWindow(DemexEditWindow::Config(
                *config_type,
            ))),

            Self::MatteoLutz => Ok(ActionRunResult::InfoWithLink(
                INFO_TEXT.to_owned(),
                "https://matteolutz.de".to_owned(),
            )),

            Self::AssignFaderToInput {
                fader_id,
                device_idx,
                input_fader_id,
            } => self.run_assign_fader_to_input(
                updatable_handler,
                input_device_handler,
                fader_id,
                device_idx,
                input_fader_id,
            ),
            Self::AssignFaderGoToInput {
                fader_id,
                device_idx,
                button_id,
            } => self.run_assign_fader_go_to_input(
                updatable_handler,
                input_device_handler,
                fader_id,
                device_idx,
                button_id,
            ),
            Self::AssignExecutorToInput {
                executor_id,
                mode,
                device_idx,
                button_id,
            } => self.run_assign_executor_to_input(
                updatable_handler,
                input_device_handler,
                executor_id,
                mode,
                device_idx,
                button_id,
            ),
            Self::AssignSelectivePresetToInput {
                preset_id,
                fixture_selector,
                device_idx,
                button_id,
            } => self.run_assign_selective_preset_to_input(
                preset_handler,
                input_device_handler,
                *preset_id,
                fixture_selector,
                fixture_selector_context,
                *device_idx,
                *button_id,
            ),
            Self::AssignFixtureSelectorToInput {
                fixture_selector,
                device_idx,
                button_id,
            } => self.run_assign_fixture_selector_to_input(
                input_device_handler,
                fixture_selector,
                device_idx,
                button_id,
            ),
            Self::AssignMacroToInput {
                action,
                device_idx,
                button_id,
            } => {
                self.run_assign_macro_to_input(input_device_handler, action, device_idx, button_id)
            }

            Self::UnassignInputButton {
                device_idx,
                button_id,
            } => self.run_unassign_input_button(input_device_handler, *device_idx, *button_id),
            Self::UnassignInputFader {
                device_idx,
                fader_id,
            } => self.run_unassign_input_fader(input_device_handler, *device_idx, *fader_id),

            unimplemented_action => Err(ActionRunError::UnimplementedAction(
                unimplemented_action.clone(),
            )),
        }
    }

    fn run_home_all(
        &self,
        fixture_handler: &mut FixtureHandler,
    ) -> Result<ActionRunResult, ActionRunError> {
        fixture_handler
            .home_all()
            .map_err(ActionRunError::FixtureHandlerError)?;

        Ok(ActionRunResult::new())
    }

    fn run_create_sequence(
        &self,
        preset_handler: &mut PresetHandler,
        id: Option<u32>,
        name: &Option<String>,
    ) -> Result<ActionRunResult, ActionRunError> {
        preset_handler
            .create_sequence(
                id.unwrap_or_else(|| preset_handler.next_sequence_id()),
                name.clone(),
            )
            .map_err(ActionRunError::PresetHandlerError)?;

        Ok(ActionRunResult::new())
    }

    fn run_create_executor(
        &self,
        updatable_handler: &mut UpdatableHandler,
        preset_handler: &PresetHandler,
        id: Option<u32>,
        name: &Option<String>,
        mode: &ExecutorCreationModeActionData,
        fixture_selector: &FixtureSelector,
        fixture_selector_context: FixtureSelectorContext,
    ) -> Result<ActionRunResult, ActionRunError> {
        let selection = fixture_selector
            .get_selection(preset_handler, fixture_selector_context)
            .map_err(ActionRunError::FixtureSelectorError)?;

        updatable_handler
            .create_executor(
                id.unwrap_or_else(|| updatable_handler.next_executor_id()),
                name.clone(),
                mode,
                selection,
            )
            .map_err(ActionRunError::UpdatableHandlerError)?;

        Ok(ActionRunResult::new())
    }

    fn run_create_macro(
        &self,
        id: Option<u32>,
        action: &Action,
        name: &Option<String>,
        preset_handler: &mut PresetHandler,
    ) -> Result<ActionRunResult, ActionRunError> {
        preset_handler
            .create_macro(
                id.unwrap_or_else(|| preset_handler.next_macro_id()),
                name.clone(),
                Box::new(action.clone()),
            )
            .map_err(ActionRunError::PresetHandlerError)?;

        Ok(ActionRunResult::new())
    }

    fn run_create_fader(
        &self,
        id: Option<u32>,
        config: &FaderCreationConfigActionData,
        name: &Option<String>,
        updatable_handler: &mut UpdatableHandler,
        preset_handler: &PresetHandler,
        fixture_selector_context: FixtureSelectorContext,
    ) -> Result<ActionRunResult, ActionRunError> {
        updatable_handler
            .create_fader(
                id.unwrap_or_else(|| updatable_handler.next_fader_id()),
                config,
                name.clone(),
                preset_handler,
                fixture_selector_context,
            )
            .map_err(ActionRunError::UpdatableHandlerError)?;

        Ok(ActionRunResult::new())
    }

    fn run_update_preset(
        &self,
        preset_handler: &mut PresetHandler,
        fixture_handler: &mut FixtureHandler,
        fixture_selector: &FixtureSelector,
        fixture_selector_context: FixtureSelectorContext,
        preset_id: FixturePresetId,
        update_mode: &UpdateModeActionData,
    ) -> Result<ActionRunResult, ActionRunError> {
        let num_updated = preset_handler
            .update_preset(
                fixture_selector,
                fixture_selector_context,
                preset_id,
                fixture_handler,
                update_mode,
            )
            .map_err(ActionRunError::PresetHandlerError)?;

        if num_updated == 0 {
            Ok(ActionRunResult::Warn(
                "No fixtures we're affected. If you're trying override existing preset data, try running with the \"override\" flag.".to_owned(),
            ))
        } else {
            Ok(ActionRunResult::new())
        }
    }

    fn run_fixture_selector(
        &self,
        fixture_selector: &FixtureSelector,
        fixture_selector_context: FixtureSelectorContext,
        preset_handler: &PresetHandler,
        fixture_handler: &FixtureHandler,
    ) -> Result<ActionRunResult, ActionRunError> {
        // flatten the fixture selector, so we don't have
        // outdated references to the previously selected fixtures
        let selection = fixture_selector
            .get_selection(preset_handler, fixture_selector_context.clone())
            .map_err(ActionRunError::FixtureSelectorError)?;

        if selection.fixtures().is_empty() {
            return Err(ActionRunError::FixtureSelectorError(
                FixtureSelectorError::NoFixturesMatched,
            ));
        }

        let unknown_fixtures = selection
            .fixtures()
            .iter()
            .filter(|f_id| !fixture_handler.has_fixture(**f_id))
            .collect::<Vec<_>>();
        if !unknown_fixtures.is_empty() {
            return Err(ActionRunError::FixtureSelectorError(
                FixtureSelectorError::SomeFixturesFailedToMatch(unknown_fixtures.len()),
            ));
        }

        Ok(ActionRunResult::UpdateSelectedFixtures(selection))
    }

    pub fn run_delete_macro(
        &self,
        (id_from, id_to): (u32, u32),
        preset_handler: &mut PresetHandler,
    ) -> Result<ActionRunResult, ActionRunError> {
        for macro_id in id_from..=id_to {
            preset_handler
                .delete_macro(macro_id)
                .map_err(ActionRunError::PresetHandlerError)?;
        }

        if id_from == id_to {
            Ok(ActionRunResult::new())
        } else {
            Ok(ActionRunResult::Info(format!(
                "Deleted {} macros",
                id_to - id_from + 1
            )))
        }
    }

    pub fn run_assign_fader_to_input(
        &self,
        updatable_handler: &UpdatableHandler,
        input_device_handler: &mut DemexInputDeviceHandler,
        fader_id: &u32,
        device_idx: &usize,
        input_fader_id: &u32,
    ) -> Result<ActionRunResult, ActionRunError> {
        let _ = updatable_handler
            .fader(*fader_id)
            .map_err(ActionRunError::UpdatableHandlerError)?;

        let device = input_device_handler
            .device_mut(*device_idx)
            .map_err(ActionRunError::InputDeviceError)?;

        if device.config.faders().get(input_fader_id).is_some() {
            return Err(ActionRunError::InputDeviceError(
                DemexInputDeviceError::FaderAlreadyAssigned(*input_fader_id),
            ));
        }

        device
            .config
            .faders_mut()
            .insert(*input_fader_id, DemexInputFader::new(*fader_id));

        Ok(ActionRunResult::new())
    }

    pub fn run_assign_fader_go_to_input(
        &self,
        updatable_handler: &UpdatableHandler,
        input_device_handler: &mut DemexInputDeviceHandler,
        fader_id: &u32,
        device_idx: &usize,
        button_id: &u32,
    ) -> Result<ActionRunResult, ActionRunError> {
        let fader = updatable_handler
            .fader(*fader_id)
            .map_err(ActionRunError::UpdatableHandlerError)?;

        match fader.config() {
            DemexFaderConfig::SequenceRuntime { .. } => {}
            _ => {
                return Err(ActionRunError::UpdatableHandlerError(
                    UpdatableHandlerError::FaderIsNotASequence(*fader_id),
                ))
            }
        }

        let device = input_device_handler
            .device_mut(*device_idx)
            .map_err(ActionRunError::InputDeviceError)?;

        if device.config.buttons().get(button_id).is_some() {
            return Err(ActionRunError::InputDeviceError(
                DemexInputDeviceError::FaderAlreadyAssigned(*button_id),
            ));
        }

        device
            .config
            .buttons_mut()
            .insert(*button_id, DemexInputButton::FaderGo(*fader_id));

        Ok(ActionRunResult::new())
    }

    pub fn run_assign_executor_to_input(
        &self,
        updatable_handler: &UpdatableHandler,
        input_device_handler: &mut DemexInputDeviceHandler,
        executor_id: &u32,
        mode: &ExecutorAssignmentModeActionData,
        device_idx: &usize,
        button_id: &u32,
    ) -> Result<ActionRunResult, ActionRunError> {
        let _ = updatable_handler.executor(*executor_id).ok_or(
            ActionRunError::UpdatableHandlerError(UpdatableHandlerError::UpdatableNotFound(
                *executor_id,
            )),
        )?;

        let device = input_device_handler
            .device_mut(*device_idx)
            .map_err(ActionRunError::InputDeviceError)?;

        if device.config.buttons().get(button_id).is_some() {
            return Err(ActionRunError::InputDeviceError(
                DemexInputDeviceError::ButtonAlreadyAssigned(*button_id),
            ));
        }

        device.config.buttons_mut().insert(
            *button_id,
            match mode {
                ExecutorAssignmentModeActionData::StartAndNext => {
                    DemexInputButton::ExecutorStartAndNext(*executor_id)
                }
                ExecutorAssignmentModeActionData::Stop => {
                    DemexInputButton::ExecutorStop(*executor_id)
                }
                ExecutorAssignmentModeActionData::Flash => {
                    DemexInputButton::ExecutorFlash(*executor_id)
                }
            },
        );

        Ok(ActionRunResult::new())
    }

    pub fn run_assign_selective_preset_to_input(
        &self,
        preset_handler: &PresetHandler,
        input_device_handler: &mut DemexInputDeviceHandler,
        preset_id: ValueOrRange<FixturePresetId>,
        fixture_selector: &Option<FixtureSelector>,
        fixture_selector_context: FixtureSelectorContext,
        device_idx: usize,
        button_id: u32,
    ) -> Result<ActionRunResult, ActionRunError> {
        let (preset_id_from, preset_id_to) = preset_id.into();

        let _ = preset_handler
            .get_preset_range(preset_id_from, preset_id_to)
            .map_err(ActionRunError::PresetHandlerError)?;

        let range = preset_id_to.preset_id - preset_id_from.preset_id + 1;

        let device = input_device_handler
            .device_mut(device_idx)
            .map_err(ActionRunError::InputDeviceError)?;

        for button_id in button_id..button_id + range {
            if device.config.buttons().get(&button_id).is_some() {
                return Err(ActionRunError::InputDeviceError(
                    DemexInputDeviceError::ButtonAlreadyAssigned(button_id),
                ));
            }
        }

        let selection = fixture_selector
            .as_ref()
            .map(|fs| fs.get_selection(preset_handler, fixture_selector_context))
            .transpose()
            .map_err(ActionRunError::FixtureSelectorError)?;

        for i in 0..range {
            let preset_id = FixturePresetId {
                feature_group_id: preset_id_from.feature_group_id,
                preset_id: preset_id_from.preset_id + i,
            };

            device.config.buttons_mut().insert(
                button_id + i,
                DemexInputButton::SelectivePreset {
                    preset_id,
                    selection: selection.clone(),
                },
            );
        }

        Ok(ActionRunResult::new())
    }

    pub fn run_assign_fixture_selector_to_input(
        &self,
        input_device_handler: &mut DemexInputDeviceHandler,
        fixture_selector: &FixtureSelector,
        device_idx: &usize,
        button_id: &u32,
    ) -> Result<ActionRunResult, ActionRunError> {
        let device = input_device_handler
            .device_mut(*device_idx)
            .map_err(ActionRunError::InputDeviceError)?;

        if device.config.buttons().get(button_id).is_some() {
            return Err(ActionRunError::InputDeviceError(
                DemexInputDeviceError::ButtonAlreadyAssigned(*button_id),
            ));
        }

        device.config.buttons_mut().insert(
            *button_id,
            DemexInputButton::FixtureSelector {
                fixture_selector: fixture_selector.clone(),
            },
        );

        Ok(ActionRunResult::new())
    }

    fn run_assign_macro_to_input(
        &self,
        input_device_handler: &mut DemexInputDeviceHandler,
        action: &Action,
        device_idx: &usize,
        button_id: &u32,
    ) -> Result<ActionRunResult, ActionRunError> {
        let device = input_device_handler
            .device_mut(*device_idx)
            .map_err(ActionRunError::InputDeviceError)?;

        if device.config.buttons().get(button_id).is_some() {
            return Err(ActionRunError::InputDeviceError(
                DemexInputDeviceError::ButtonAlreadyAssigned(*button_id),
            ));
        }

        device.config.buttons_mut().insert(
            *button_id,
            DemexInputButton::Macro {
                action: action.clone(),
            },
        );

        Ok(ActionRunResult::new())
    }

    fn run_unassign_input_button(
        &self,
        input_device_handler: &mut DemexInputDeviceHandler,
        device_idx: usize,
        button_id: u32,
    ) -> Result<ActionRunResult, ActionRunError> {
        let device = input_device_handler
            .device_mut(device_idx)
            .map_err(ActionRunError::InputDeviceError)?;

        if device.config.buttons_mut().remove(&button_id).is_none() {
            return Err(ActionRunError::InputDeviceError(
                DemexInputDeviceError::ButtonNotAssigned(button_id),
            ));
        }

        Ok(ActionRunResult::new())
    }

    fn run_unassign_input_fader(
        &self,
        input_device_handler: &mut DemexInputDeviceHandler,
        device_idx: usize,
        fader_id: u32,
    ) -> Result<ActionRunResult, ActionRunError> {
        let device = input_device_handler
            .device_mut(device_idx)
            .map_err(ActionRunError::InputDeviceError)?;

        if device.config.faders_mut().remove(&fader_id).is_none() {
            return Err(ActionRunError::InputDeviceError(
                DemexInputDeviceError::FaderNotAssigned(fader_id),
            ));
        }

        Ok(ActionRunResult::new())
    }
}
