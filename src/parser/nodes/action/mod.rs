use std::ops::RangeInclusive;

use functions::{
    assign_function::{AssignButtonArgs, AssignFaderArgs},
    create_function::{CreateExecutorArgs, CreateFaderArgs, CreateMacroArgs, CreateSequenceArgs},
    delete_function::DeleteArgs,
    record_function::{RecordGroupArgs, RecordPresetArgs, RecordSequenceCueArgs},
    rename_function::RenameObjectArgs,
    set_function::{SetFeatureValueArgs, SetFixturePresetArgs},
    update_function::{UpdatePresetArgs, UpdateSequenceCueArgs},
    FunctionArgs,
};
use serde::{Deserialize, Serialize};

use crate::{
    fixture::{
        handler::FixtureHandler, presets::PresetHandler, timing::TimingHandler,
        updatables::UpdatableHandler,
    },
    input::{error::DemexInputDeviceError, DemexInputDeviceHandler},
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

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
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

impl<T: Copy> From<ValueOrRange<T>> for RangeInclusive<T> {
    fn from(value: ValueOrRange<T>) -> Self {
        match value {
            ValueOrRange::Single(single) => single..=single,
            ValueOrRange::Thru(from, to) => from..=to,
        }
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ConfigTypeActionData {
    Output,
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

    Rename(RenameObjectArgs),

    CreateSequence(CreateSequenceArgs),
    CreateExecutor(CreateExecutorArgs),
    CreateFader(CreateFaderArgs),
    CreateMacro(CreateMacroArgs),

    UpdatePreset(UpdatePresetArgs),
    UpdateSequenceCue(UpdateSequenceCueArgs),

    Delete(DeleteArgs),

    Edit(Object),

    // Assign
    AssignButton(AssignButtonArgs),
    AssignFader(AssignFaderArgs),

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
        timing_handler: &mut TimingHandler,
    ) -> Result<ActionRunResult, ActionRunError> {
        match self {
            // Set
            Self::SetFeatureValue(args) => args.run(
                fixture_handler,
                preset_handler,
                fixture_selector_context,
                updatable_handler,
                input_device_handler,
                timing_handler,
            ),
            Self::SetFixturePreset(args) => args.run(
                fixture_handler,
                preset_handler,
                fixture_selector_context,
                updatable_handler,
                input_device_handler,
                timing_handler,
            ),

            // Home
            Self::Home(homeable_object) => homeable_object.run_home(
                preset_handler,
                fixture_handler,
                updatable_handler,
                fixture_selector_context,
            ),

            Self::HomeAll => self.run_home_all(fixture_handler),

            // Record
            Self::RecordPreset(args) => args.run(
                fixture_handler,
                preset_handler,
                fixture_selector_context,
                updatable_handler,
                input_device_handler,
                timing_handler,
            ),
            Self::RecordGroup2(args) => args.run(
                fixture_handler,
                preset_handler,
                fixture_selector_context,
                updatable_handler,
                input_device_handler,
                timing_handler,
            ),
            Self::RecordSequenceCue(args) => args.run(
                fixture_handler,
                preset_handler,
                fixture_selector_context,
                updatable_handler,
                input_device_handler,
                timing_handler,
            ),

            // Rename
            Self::Rename(args) => args.run(
                fixture_handler,
                preset_handler,
                fixture_selector_context,
                updatable_handler,
                input_device_handler,
                timing_handler,
            ),

            // Create
            Self::CreateSequence(args) => args.run(
                fixture_handler,
                preset_handler,
                fixture_selector_context,
                updatable_handler,
                input_device_handler,
                timing_handler,
            ),
            Self::CreateExecutor(args) => args.run(
                fixture_handler,
                preset_handler,
                fixture_selector_context,
                updatable_handler,
                input_device_handler,
                timing_handler,
            ),
            Self::CreateMacro(args) => args.run(
                fixture_handler,
                preset_handler,
                fixture_selector_context,
                updatable_handler,
                input_device_handler,
                timing_handler,
            ),
            Self::CreateFader(args) => args.run(
                fixture_handler,
                preset_handler,
                fixture_selector_context,
                updatable_handler,
                input_device_handler,
                timing_handler,
            ),

            // Update
            Self::UpdatePreset(args) => args.run(
                fixture_handler,
                preset_handler,
                fixture_selector_context,
                updatable_handler,
                input_device_handler,
                timing_handler,
            ),
            Self::UpdateSequenceCue(args) => args.run(
                fixture_handler,
                preset_handler,
                fixture_selector_context,
                updatable_handler,
                input_device_handler,
                timing_handler,
            ),

            // Delete
            Self::Delete(args) => args.run(
                fixture_handler,
                preset_handler,
                fixture_selector_context,
                updatable_handler,
                input_device_handler,
                timing_handler,
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

            Self::AssignFader(args) => args.run(
                fixture_handler,
                preset_handler,
                fixture_selector_context,
                updatable_handler,
                input_device_handler,
                timing_handler,
            ),

            Self::AssignButton(args) => args.run(
                fixture_handler,
                preset_handler,
                fixture_selector_context,
                updatable_handler,
                input_device_handler,
                timing_handler,
            ),

            Self::UnassignInputButton {
                device_idx,
                button_id,
            } => self.run_unassign_input_button(input_device_handler, *device_idx, *button_id),
            Self::UnassignInputFader {
                device_idx,
                fader_id,
            } => self.run_unassign_input_fader(input_device_handler, *device_idx, *fader_id),

            #[allow(unreachable_patterns)]
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
            .home_all(true)
            .map_err(ActionRunError::FixtureHandlerError)?;

        Ok(ActionRunResult::new())
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
