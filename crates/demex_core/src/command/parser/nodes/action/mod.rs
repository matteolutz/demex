use std::{ops::RangeInclusive, time};

use functions::{
    FunctionArgs,
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
};
use serde::{Deserialize, Serialize};
use strum::EnumIter;

use crate::{
    command::parser::nodes::action::functions::{
        move_function::MoveArgs, set_function::ObjectSetPropertyArgs,
    },
    event::DemexEvent,
    state::fixture_state_handler::FixtureStateHandler,
    utils::serde::approx_instant,
};

use crate::{
    input::{DemexInputDeviceHandler, error::DemexInputDeviceError},
    patch::Patch,
    presets::PresetHandler,
    selection::FixtureSelection,
    timing::TimingHandler,
    updatables::UpdatableHandler,
};

use self::{error::ActionRunError, result::ActionRunResult};

use super::{
    fixture_selector::{FixtureSelector, FixtureSelectorContext, FixtureSelectorError},
    object::{HomeableObject, Object},
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
    Ui,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ActionIssuer {
    Command,
    Macro,
    Ui,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeferredAction {
    pub action: Action,

    #[serde(with = "approx_instant")]
    pub issued_at: time::Instant,

    pub issuer: ActionIssuer,
}

impl DeferredAction {
    pub fn run(
        &self,
        fixture_handler: &mut FixtureStateHandler,
        preset_handler: &mut PresetHandler,
        fixture_selector_context: FixtureSelectorContext,
        updatable_handler: &mut UpdatableHandler,
        input_device_handler: &mut DemexInputDeviceHandler,
        timing_handler: &mut TimingHandler,
        patch: &Patch,
    ) -> Result<ActionRunResult, ActionRunError> {
        self.action.run(
            fixture_handler,
            preset_handler,
            fixture_selector_context,
            updatable_handler,
            input_device_handler,
            timing_handler,
            patch,
            self.issued_at,
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum Action {
    // Set
    SetFeatureValue(SetFeatureValueArgs),
    SetFixturePreset(SetFixturePresetArgs),
    ObjectSetProperty(ObjectSetPropertyArgs),

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

    // Move
    Move(MoveArgs),

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
    GrandEtc,

    SetFixtureSelection(Option<FixtureSelection>),
    SetFixtureSelectionWing(usize),

    ExecutorGo(ExecutorGoArgs),
    ExecutorStop(ExecutorStopArgs),
    ExecutorSetFaderValue(u32, f32),

    Lock,

    #[default]
    MatteoLutz,
}

impl Action {
    pub fn run(
        &self,
        fixture_handler: &mut FixtureStateHandler,
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
            Self::ObjectSetProperty(args) => args.run(
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
            Self::Home(homeable_object) => homeable_object.home(
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

            // Move
            Self::Move(args) => args.run(
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

            Self::ClearAll => Ok(ActionRunResult::UpdateFixtureSelection(None)),
            Self::FixtureSelector(fixture_selector) => self.run_fixture_selector(
                fixture_selector,
                fixture_selector_context,
                preset_handler,
                patch,
            ),
            Self::Test(_) => Ok(ActionRunResult::new()),
            Self::Save => Ok(ActionRunResult::Save),

            Self::Nuzul => Ok(ActionRunResult::Info("Going down...".to_owned())),
            Self::Sueud => Ok(ActionRunResult::Info("Going up...".to_owned())),
            Self::GrandEtc => Ok(ActionRunResult::Warn("Ha ha ha, very funny".to_owned())),

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

            Self::SetFixtureSelection(selection) => {
                Ok(ActionRunResult::UpdateFixtureSelection(selection.clone()))
            }
            Self::ExecutorGo(args) => args.run(
                issued_at,
                fixture_handler,
                preset_handler,
                fixture_selector_context,
                updatable_handler,
                input_device_handler,
                timing_handler,
                patch,
            ),
            Self::ExecutorStop(args) => args.run(
                issued_at,
                fixture_handler,
                preset_handler,
                fixture_selector_context,
                updatable_handler,
                input_device_handler,
                timing_handler,
                patch,
            ),
            Self::ExecutorSetFaderValue(executor_id, fader_value) => {
                let executor = updatable_handler
                    .executor_mut(*executor_id)
                    .map_err(ActionRunError::UpdatableHandlerError)?;
                executor.set_value(
                    *fader_value,
                    fixture_handler,
                    preset_handler,
                    issued_at.elapsed().as_secs_f32(),
                );

                Ok(ActionRunResult::event(
                    DemexEvent::ExecutorFaderValueChanged(*executor_id),
                ))
            }

            Self::Lock => Ok(ActionRunResult::Lock),

            #[allow(unreachable_patterns)]
            unimplemented_action => Err(ActionRunError::UnimplementedAction(
                unimplemented_action.clone(),
            )),
        }
    }

    fn run_home_all(
        &self,
        fixture_handler: &mut FixtureStateHandler,
    ) -> Result<ActionRunResult, ActionRunError> {
        fixture_handler
            .home_all(true)
            .map_err(ActionRunError::FixtureError)?;

        Ok(ActionRunResult::new())
    }

    fn run_fixture_selector(
        &self,
        fixture_selector: &FixtureSelector,
        fixture_selector_context: FixtureSelectorContext,
        preset_handler: &PresetHandler,
        patch: &Patch,
    ) -> Result<ActionRunResult, ActionRunError> {
        // flatten the fixture selector, so we don't have
        // outdated references to the previously selected fixtures
        let mut selection = fixture_selector
            .get_selection(preset_handler, fixture_selector_context.clone())
            .map_err(ActionRunError::FixtureSelectorError)?;

        if selection.fixtures().is_empty() {
            return Err(ActionRunError::FixtureSelectorError(
                FixtureSelectorError::NoFixturesMatched,
            ));
        }

        selection.retain(|id| patch.fixture(*id).is_ok());

        Ok(ActionRunResult::UpdateFixtureSelection(Some(selection)))
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
