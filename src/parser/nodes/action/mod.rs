use std::{ops::RangeInclusive, time};

use functions::{
    assign_function::{AssignButtonArgs, AssignFaderArgs},
    create_function::{
        CreateEffectPresetArgs, CreateExecutorArgs, CreateMacroArgs, CreateSequenceArgs,
    },
    delete_function::DeleteArgs,
    go_function::ExecutorGoArgs,
    recall_function::RecallSequenceCueArgs,
    record_function::{
        RecordGroupArgs, RecordPresetArgs, RecordSequenceCueArgs, RecordSequenceCueShorthandArgs,
    },
    rename_function::RenameObjectArgs,
    set_function::{SetFeatureValueArgs, SetFixturePresetArgs},
    stop_function::ExecutorStopArgs,
    update_function::{UpdatePresetArgs, UpdateSequenceCueArgs},
    FunctionArgs,
};
use serde::{Deserialize, Serialize};
use strum::EnumIter;

use crate::utils::serde::approx_instant;

use crate::{
    fixture::{
        handler::FixtureHandler, patch::Patch, presets::PresetHandler, selection::FixtureSelection,
        timing::TimingHandler, updatables::UpdatableHandler,
    },
    input::{error::DemexInputDeviceError, DemexInputDeviceHandler},
};

use self::{error::ActionRunError, result::ActionRunResult};

use super::{
    fixture_selector::{FixtureSelector, FixtureSelectorContext, FixtureSelectorError},
    object::{HomeableObject, Object, ObjectTrait},
};

pub mod error;
pub mod functions;
pub mod queue;
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

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, EnumIter)]
pub enum ConfigTypeActionData {
    Output,
    Patch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeferredAction {
    pub action: Action,

    #[serde(with = "approx_instant")]
    pub issued_at: time::Instant,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum Action {
    // Set
    SetFeatureValue(SetFeatureValueArgs),
    SetFixturePreset(SetFixturePresetArgs),

    // Home
    Home(HomeableObject),
    HomeAll,

    // Record
    RecordPreset(RecordPresetArgs),
    RecordGroup2(RecordGroupArgs),
    RecordSequenceCue(RecordSequenceCueArgs),
    RecordSequenceCueShorthand(RecordSequenceCueShorthandArgs),

    // Reanme
    Rename(RenameObjectArgs),

    // Create
    CreateSequence(CreateSequenceArgs),
    CreateExecutor(CreateExecutorArgs),
    CreateMacro(CreateMacroArgs),
    CreateEffectPreset(CreateEffectPresetArgs),

    // Update
    UpdatePreset(UpdatePresetArgs),
    UpdateSequenceCue(UpdateSequenceCueArgs),

    // Recall
    RecallSequenceCue(RecallSequenceCueArgs),

    // Delete
    Delete(DeleteArgs),

    // Edit
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

    // Internal
    InternalSetFixtureSelection(Option<FixtureSelection>),
    InternalExecutorGo(ExecutorGoArgs),
    InternalExecutorStop(ExecutorStopArgs),

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
        patch: &Patch,
        issued_at: time::Instant,
    ) -> Result<ActionRunResult, ActionRunError> {
        match self {
            // Set
            Self::SetFeatureValue(args) => args.run(
                issued_at,
                fixture_handler,
                preset_handler,
                fixture_selector_context,
                updatable_handler,
                input_device_handler,
                timing_handler,
                patch,
            ),
            Self::SetFixturePreset(args) => args.run(
                issued_at,
                fixture_handler,
                preset_handler,
                fixture_selector_context,
                updatable_handler,
                input_device_handler,
                timing_handler,
                patch,
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
                issued_at,
                fixture_handler,
                preset_handler,
                fixture_selector_context,
                updatable_handler,
                input_device_handler,
                timing_handler,
                patch,
            ),
            Self::RecordGroup2(args) => args.run(
                issued_at,
                fixture_handler,
                preset_handler,
                fixture_selector_context,
                updatable_handler,
                input_device_handler,
                timing_handler,
                patch,
            ),
            Self::RecordSequenceCue(args) => args.run(
                issued_at,
                fixture_handler,
                preset_handler,
                fixture_selector_context,
                updatable_handler,
                input_device_handler,
                timing_handler,
                patch,
            ),
            Self::RecordSequenceCueShorthand(args) => args.run(
                issued_at,
                fixture_handler,
                preset_handler,
                fixture_selector_context,
                updatable_handler,
                input_device_handler,
                timing_handler,
                patch,
            ),

            // Rename
            Self::Rename(args) => args.run(
                issued_at,
                fixture_handler,
                preset_handler,
                fixture_selector_context,
                updatable_handler,
                input_device_handler,
                timing_handler,
                patch,
            ),

            // Create
            Self::CreateSequence(args) => args.run(
                issued_at,
                fixture_handler,
                preset_handler,
                fixture_selector_context,
                updatable_handler,
                input_device_handler,
                timing_handler,
                patch,
            ),
            Self::CreateExecutor(args) => args.run(
                issued_at,
                fixture_handler,
                preset_handler,
                fixture_selector_context,
                updatable_handler,
                input_device_handler,
                timing_handler,
                patch,
            ),
            Self::CreateMacro(args) => args.run(
                issued_at,
                fixture_handler,
                preset_handler,
                fixture_selector_context,
                updatable_handler,
                input_device_handler,
                timing_handler,
                patch,
            ),
            Self::CreateEffectPreset(args) => args.run(
                issued_at,
                fixture_handler,
                preset_handler,
                fixture_selector_context,
                updatable_handler,
                input_device_handler,
                timing_handler,
                patch,
            ),

            // Update
            Self::UpdatePreset(args) => args.run(
                issued_at,
                fixture_handler,
                preset_handler,
                fixture_selector_context,
                updatable_handler,
                input_device_handler,
                timing_handler,
                patch,
            ),
            Self::UpdateSequenceCue(args) => args.run(
                issued_at,
                fixture_handler,
                preset_handler,
                fixture_selector_context,
                updatable_handler,
                input_device_handler,
                timing_handler,
                patch,
            ),

            Self::RecallSequenceCue(args) => args.run(
                issued_at,
                fixture_handler,
                preset_handler,
                fixture_selector_context,
                updatable_handler,
                input_device_handler,
                timing_handler,
                patch,
            ),

            // Delete
            Self::Delete(args) => args.run(
                issued_at,
                fixture_handler,
                preset_handler,
                fixture_selector_context,
                updatable_handler,
                input_device_handler,
                timing_handler,
                patch,
            ),

            #[cfg(feature = "ui")]
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

            #[cfg(feature = "ui")]
            Self::Config(config_type) => Ok(ActionRunResult::EditWindow(
                crate::ui::window::edit::DemexEditWindow::Config(*config_type),
            )),

            #[cfg(feature = "ui")]
            Self::MatteoLutz => Ok(ActionRunResult::InfoWithLink(
                crate::ui::constants::INFO_TEXT.to_owned(),
                "https://matteolutz.de".to_owned(),
            )),

            Self::AssignFader(args) => args.run(
                issued_at,
                fixture_handler,
                preset_handler,
                fixture_selector_context,
                updatable_handler,
                input_device_handler,
                timing_handler,
                patch,
            ),

            Self::AssignButton(args) => args.run(
                issued_at,
                fixture_handler,
                preset_handler,
                fixture_selector_context,
                updatable_handler,
                input_device_handler,
                timing_handler,
                patch,
            ),

            Self::UnassignInputButton {
                device_idx,
                button_id,
            } => self.run_unassign_input_button(input_device_handler, *device_idx, *button_id),
            Self::UnassignInputFader {
                device_idx,
                fader_id,
            } => self.run_unassign_input_fader(input_device_handler, *device_idx, *fader_id),

            Self::InternalSetFixtureSelection(selection) => {
                Ok(ActionRunResult::UpdateSelectedFixtures(selection.clone()))
            }
            Self::InternalExecutorGo(args) => args.run(
                issued_at,
                fixture_handler,
                preset_handler,
                fixture_selector_context,
                updatable_handler,
                input_device_handler,
                timing_handler,
                patch,
            ),
            Self::InternalExecutorStop(args) => args.run(
                issued_at,
                fixture_handler,
                preset_handler,
                fixture_selector_context,
                updatable_handler,
                input_device_handler,
                timing_handler,
                patch,
            ),

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

        Ok(ActionRunResult::UpdateSelectedFixtures(Some(selection)))
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
