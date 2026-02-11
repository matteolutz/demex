use std::{ops::RangeInclusive, time};

use demex_dmx::DemexDmxOutputConfig;
use functions::{
    FunctionDelegate,
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
    set_function::{SetAttributeValueArgs, SetFixturePresetArgs},
    stop_function::ExecutorStopArgs,
    update_function::{UpdatePresetArgs, UpdateSequenceCueArgs},
};
use serde::{Deserialize, Serialize};
use strum::EnumIter;

use crate::{
    command::parser::nodes::action::functions::{
        effect_function::{KeyframeEffectApplyPresetArgs, KeyframeEffectUpdateArgs},
        go_function::ExecutorGoOutArgs,
        move_function::MoveArgs,
        recall_function::RecallEffectKeyframeArgs,
        set_function::{CueSetTriggerArgs, ObjectSetPropertyArgs, SetAttributeChannelSetArgs},
        speedmaster_functions::SpeedMasterTapArgs,
        start_function::ExecutorStartArgs,
        stomp_function::ExecutorStompArgs,
        update_function::UpdatePresetGlobalArgs,
    },
    event::{DemexEvent, FixtureSelectionWithGroup, list::DemexEventList},
    fixture::FixturePath,
    input::control::DemexInputDeviceControlUnassignment,
    master::MasterHandler,
    state::fixture_state_handler::FixtureStateHandler,
    utils::serde::approx_instant,
};

use crate::{
    patch::Patch, presets::PresetHandler, selection::FixtureSelection, timing::TimingHandler,
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

pub struct DeferredActionRunArgs<'a> {
    pub patch: &'a Patch,

    pub fixture_handler: &'a mut FixtureStateHandler,
    pub preset_handler: &'a mut PresetHandler,
    pub updatable_handler: &'a mut UpdatableHandler,
    pub timing_handler: &'a mut TimingHandler,
    pub master_handler: &'a mut MasterHandler,

    pub fixture_selector_context: FixtureSelectorContext<'a>,

    pub event_list: &'a mut DemexEventList,
}

impl<'a> DeferredActionRunArgs<'a> {
    pub fn into_action(self, issued_at: time::Instant) -> ActionRunArgs<'a> {
        ActionRunArgs {
            issued_at,
            patch: self.patch,
            fixture_handler: self.fixture_handler,
            preset_handler: self.preset_handler,
            updatable_handler: self.updatable_handler,
            timing_handler: self.timing_handler,
            master_handler: self.master_handler,
            fixture_selector_context: self.fixture_selector_context,
            event_list: self.event_list,
        }
    }
}

pub struct ActionRunArgs<'a> {
    pub issued_at: time::Instant,

    pub patch: &'a Patch,

    pub fixture_handler: &'a mut FixtureStateHandler,
    pub preset_handler: &'a mut PresetHandler,
    pub updatable_handler: &'a mut UpdatableHandler,
    pub timing_handler: &'a mut TimingHandler,
    pub master_handler: &'a mut MasterHandler,

    pub fixture_selector_context: FixtureSelectorContext<'a>,

    pub event_list: &'a mut DemexEventList,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ValueOrRange<T> {
    Single(T),
    Thru(T, T),
}

impl<T> From<T> for ValueOrRange<T> {
    fn from(value: T) -> Self {
        ValueOrRange::Single(value)
    }
}

impl<T> From<(T, T)> for ValueOrRange<T> {
    fn from((from, to): (T, T)) -> Self {
        ValueOrRange::Thru(from, to)
    }
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
    InputDevice,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeferredAction {
    pub action: Action,

    #[serde(with = "approx_instant")]
    pub issued_at: time::Instant,

    pub issuer: ActionIssuer,
}

impl DeferredAction {
    pub fn run(&self, args: DeferredActionRunArgs) -> Result<ActionRunResult, ActionRunError> {
        self.action.run(args.into_action(self.issued_at))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum Action {
    // Set
    SetAttributeValue(SetAttributeValueArgs),
    SetAttributeChannlSet(SetAttributeChannelSetArgs),
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
    UpdatePresetGlobal(UpdatePresetGlobalArgs),
    UpdateSequenceCue(UpdateSequenceCueArgs),

    // Recall
    RecallSequenceCue(RecallSequenceCueArgs),
    RecallEffectKeyframe(RecallEffectKeyframeArgs),

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

    Highlight(Option<FixtureSelector>),
    Unhighlight,

    ClearAll,
    Save,
    Test(String),

    Config(ConfigTypeActionData),

    Nuzul,
    Sueud,
    GrandEtc,

    SetFixtureSelection(Option<FixtureSelection>),
    AddFixturesToSelection(Vec<FixturePath>),
    SetFixtureSelectionWing(usize),

    ExecutorStart(ExecutorStartArgs),
    ExecutorStomp(ExecutorStompArgs),
    ExecutorGo(ExecutorGoArgs),
    ExecutorGoOut(ExecutorGoOutArgs),
    ExecutorStop(ExecutorStopArgs),
    ExecutorSetFaderValue(u32, f32),

    CueSetTrigger(CueSetTriggerArgs),

    SpeedMasterSetBpm(u32, f32),
    SpeedMasterTap(SpeedMasterTapArgs),

    KeyframeEffectUpdate(KeyframeEffectUpdateArgs),
    KeyframeEffectApplyPreset(KeyframeEffectApplyPresetArgs),

    GrandmasterSetValue(f32),
    GroupmasterSetValue(u32, f32),

    RunMacro(u32),

    UpdateOutputConfigs(Vec<DemexDmxOutputConfig>),

    Lock,

    #[default]
    MatteoLutz,
}

impl Action {
    pub fn run(&self, args: ActionRunArgs) -> Result<ActionRunResult, ActionRunError> {
        match self {
            // Set
            Self::SetAttributeValue(fun) => fun.run(args),
            Self::SetAttributeChannlSet(fun) => fun.run(args),
            Self::SetFixturePreset(fun) => fun.run(args),
            Self::ObjectSetProperty(fun) => fun.run(args),

            // Home
            Self::Home(homeable_object) => homeable_object.home(args),

            Self::HomeAll => self.run_home_all(args.fixture_handler),

            // Record
            Self::RecordPreset(fun) => fun.run(args),
            Self::RecordGroup2(fun) => fun.run(args),
            Self::RecordSequenceCue(fun) => fun.run(args),
            Self::RecordSequenceCueShorthand(fun) => fun.run(args),

            // Rename
            Self::Rename(fun) => fun.run(args),

            // Create
            Self::CreateSequence(fun) => fun.run(args),
            Self::CreateExecutor(fun) => fun.run(args),
            Self::CreateMacro(fun) => fun.run(args),
            Self::CreateEffectPreset(fun) => fun.run(args),

            // Update
            Self::UpdatePreset(fun) => fun.run(args),
            Self::UpdatePresetGlobal(fun) => fun.run(args),
            Self::UpdateSequenceCue(fun) => fun.run(args),

            Self::RecallSequenceCue(fun) => fun.run(args),
            Self::RecallEffectKeyframe(fun) => fun.run(args),

            // Delete
            Self::Delete(fun) => fun.run(args),

            // Move
            Self::Move(fun) => fun.run(args),

            Self::ClearAll => Ok(ActionRunResult::UpdateFixtureSelection(None)),
            Self::FixtureSelector(fixture_selector) => {
                self.run_fixture_selector(fixture_selector.clone(), args)
            }
            Self::Highlight(fixture_selector) => {
                self.run_highlight(fixture_selector.as_ref(), args)
            }
            Self::Unhighlight => Ok(ActionRunResult::UpdateHighlight(None)),
            Self::Test(_) => Ok(ActionRunResult::new()),
            Self::Save => Ok(ActionRunResult::Save),

            Self::Nuzul => Ok(ActionRunResult::Info("Going down...".to_owned())),
            Self::Sueud => Ok(ActionRunResult::Info("Going up...".to_owned())),
            Self::GrandEtc => Ok(ActionRunResult::Warn("Ha ha ha, very funny".to_owned())),

            Self::UpdateOutputConfigs(configs) => {
                let mut patch = args.patch.clone();
                *patch.output_configs_mut() = configs.clone();
                Ok(ActionRunResult::UpdatePatch(patch))
            }

            Self::AssignFader(fun) => fun.run(args),

            Self::AssignButton(fun) => fun.run(args),

            Self::UnassignInputButton {
                device_idx,
                button_id,
            } => self.run_unassign_input_button(*device_idx, *button_id),
            Self::UnassignInputFader {
                device_idx,
                fader_id,
            } => self.run_unassign_input_fader(*device_idx, *fader_id),

            Self::SetFixtureSelection(selection) => Ok(ActionRunResult::UpdateFixtureSelection(
                selection.clone().map(|sel| sel.into()),
            )),
            Self::AddFixturesToSelection(fixtures) => {
                if fixtures.is_empty() {
                    Ok(ActionRunResult::new())
                } else {
                    let selection = match args.fixture_selector_context.current_fixture() {
                        None => fixtures.clone().into(),
                        Some(selection) => selection.clone().with_additional_fixtures(fixtures),
                    };
                    Ok(ActionRunResult::UpdateFixtureSelection(Some(
                        selection.into(),
                    )))
                }
            }
            Self::ExecutorStomp(fun) => fun.run(args),
            Self::ExecutorStart(fun) => fun.run(args),
            Self::ExecutorGo(fun) => fun.run(args),
            Self::ExecutorGoOut(fun) => fun.run(args),
            Self::ExecutorStop(fun) => fun.run(args),
            Self::ExecutorSetFaderValue(executor_id, fader_value) => {
                let executor = args
                    .updatable_handler
                    .executor_mut(*executor_id)
                    .map_err(ActionRunError::UpdatableHandlerError)?;
                executor.set_value(
                    *fader_value,
                    args.fixture_handler,
                    args.preset_handler,
                    args.issued_at.elapsed().as_secs_f32(),
                    args.event_list,
                );

                Ok(ActionRunResult::Default)
            }

            Self::CueSetTrigger(fun) => fun.run(args),

            Self::SpeedMasterSetBpm(speed_master_id, bpm) => {
                let speed_master = args
                    .timing_handler
                    .get_speed_master_value_mut(*speed_master_id)
                    .map_err(ActionRunError::TimingHandlerError)?;

                *speed_master.bpm_mut() = *bpm;
                args.event_list
                    .push(DemexEvent::SpeedmasterFaderValueChanged {
                        speed_master_id: *speed_master_id,
                        bpm: *bpm,
                    });
                Ok(ActionRunResult::new())
            }
            Self::SpeedMasterTap(fun) => fun.run(args),

            Self::KeyframeEffectUpdate(fun) => fun.run(args),
            Self::KeyframeEffectApplyPreset(fun) => fun.run(args),

            Self::GrandmasterSetValue(value) => {
                args.master_handler.set_grand_master(*value);
                Ok(ActionRunResult::new())
            }
            Self::GroupmasterSetValue(group_id, value) => {
                args.master_handler
                    .set_groupmaster_value(*group_id, *value, args.preset_handler);
                Ok(ActionRunResult::new())
            }

            Self::RunMacro(macro_id) => {
                let mmacro = args
                    .preset_handler
                    .get_macro(*macro_id)
                    .map_err(ActionRunError::PresetHandlerError)?;
                mmacro.action().clone().run(args)
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
        fixture_selector: FixtureSelector,
        args: ActionRunArgs,
    ) -> Result<ActionRunResult, ActionRunError> {
        let group_id = fixture_selector.try_as_group_id();

        // flatten the fixture selector, so we don't have
        // outdated references to the previously selected fixtures
        let mut selection = fixture_selector
            .get_selection(args.preset_handler, args.fixture_selector_context.clone())
            .map_err(ActionRunError::FixtureSelectorError)?;

        if selection.fixtures().is_empty() {
            return Err(ActionRunError::FixtureSelectorError(
                FixtureSelectorError::NoFixturesMatched,
            ));
        }

        selection.retain(|path| args.patch.fixture(path).is_ok());

        Ok(ActionRunResult::UpdateFixtureSelection(Some(
            FixtureSelectionWithGroup::with_group(selection, group_id),
        )))
    }

    fn run_highlight(
        &self,
        fixture_selector: Option<&FixtureSelector>,
        args: ActionRunArgs,
    ) -> Result<ActionRunResult, ActionRunError> {
        let fixture_selector = fixture_selector
            .cloned()
            .unwrap_or_else(|| FixtureSelector::current_fixtures_selected());

        let group_id = fixture_selector.try_as_group_id();

        // flatten the fixture selector, so we don't have
        // outdated references to the previously selected fixtures
        let mut selection = fixture_selector
            .get_selection(args.preset_handler, args.fixture_selector_context.clone())
            .map_err(ActionRunError::FixtureSelectorError)?;

        if selection.fixtures().is_empty() {
            return Err(ActionRunError::FixtureSelectorError(
                FixtureSelectorError::NoFixturesMatched,
            ));
        }

        selection.retain(|path| args.patch.fixture(path).is_ok());

        Ok(ActionRunResult::UpdateHighlight(Some(
            FixtureSelectionWithGroup::with_group(selection, group_id),
        )))
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
        device_idx: usize,
        button_id: u32,
    ) -> Result<ActionRunResult, ActionRunError> {
        Ok(ActionRunResult::Unassign(
            DemexInputDeviceControlUnassignment::Button {
                device_idx,
                button_id,
            },
        ))
    }

    fn run_unassign_input_fader(
        &self,
        device_idx: usize,
        fader_id: u32,
    ) -> Result<ActionRunResult, ActionRunError> {
        Ok(ActionRunResult::Unassign(
            DemexInputDeviceControlUnassignment::Fader {
                device_idx,
                fader_id,
            },
        ))
    }
}
