use std::{any::Any, fmt::Display};

use serde::{Deserialize, Serialize};

use crate::{
    event::{DemexEvent, list::DemexEventList},
    pool::PoolType,
    presets::{PresetHandler, error::PresetHandlerError, preset::FixturePresetId},
    selection::FixtureSelection,
    sequence::cue::CueIdx,
    state::fixture_state_handler::FixtureStateHandler,
    updatables::UpdatableHandler,
};

use super::{
    action::{Action, error::ActionRunError, result::ActionRunResult},
    fixture_selector::{FixtureSelector, FixtureSelectorContext},
};

pub mod error;
pub use error::*;

#[macro_export]
macro_rules! implement_set_property {
    (
        for $object_type:ident with $property_enum:ty,
        $(
            $property_variant:ident => $($field:ident).+ as $field_type:ty
        ),*
    ) => {
        mod property {
            use super::*;
            use std::any::Any;
            use crate::command::parser::nodes::action::{result::ActionRunResult, error::ActionRunError};
            use crate::command::parser::nodes::object::{ObjectSetPropertyDelegate, error::ObjectError};

            impl ObjectSetPropertyDelegate for $object_type {

                type ObjectSetPropertyKeyType = $property_enum;

                fn set_property(
                    &mut self,
                    key: Self::ObjectSetPropertyKeyType,
                    value: String,
                ) -> Result<ActionRunResult, ActionRunError> {
                    match key {
                        $(
                            <$property_enum>::$property_variant => {
                                self.$($field).+ = value.parse::<$field_type>().map_err(|_| {
                                    ActionRunError::ObjectError(ObjectError::ObjectSetValueInvalid(
                                        key.to_string(),
                                        value,
                                    ))
                                })?
                            }
                        ),*
                    };

                    Ok(ActionRunResult::new())
                }

                fn set_property_any(
                    &mut self,
                    key: Self::ObjectSetPropertyKeyType,
                    value: Box<dyn Any>,
                ) -> Result<ActionRunResult, ActionRunError> {
                    match key {
                        $(
                            <$property_enum>::$property_variant => {
                                self.$($field).+ = *(value.downcast::<$field_type>().map_err(|value| {
                                    ActionRunError::ObjectError(ObjectError::ObjectSetValueInvalidAny(
                                        key.to_string(),
                                        value,
                                    ))
                                }))?
                            }
                        ),*
                    };

                    Ok(ActionRunResult::new())
                }

                fn get_property(
                    &self,
                    key: Self::ObjectSetPropertyKeyType,
                ) -> Result<String, ActionRunError> {
                    match key {
                        $(
                            <$property_enum>::$property_variant => {
                                Ok(self.$($field).+.to_string())
                            }
                        ),*
                    }
                }
            }
        }
    };
}

pub struct EmptyObjectSetKeyType {}
impl std::str::FromStr for EmptyObjectSetKeyType {
    type Err = ();

    fn from_str(_: &str) -> Result<Self, Self::Err> {
        Err(())
    }
}

pub trait ObjectSetPropertyDelegate: 'static + Sized {
    type ObjectSetPropertyKeyType: std::str::FromStr + std::fmt::Display;

    fn set_property(
        &mut self,
        key: Self::ObjectSetPropertyKeyType,
        value: String,
    ) -> Result<ActionRunResult, ActionRunError>;

    fn set_property_any(
        &mut self,
        key: Self::ObjectSetPropertyKeyType,
        value: Box<dyn Any>,
    ) -> Result<ActionRunResult, ActionRunError>;

    fn get_property(&self, key: Self::ObjectSetPropertyKeyType) -> Result<String, ActionRunError>;
}

trait ObjectSetPropertyStringDelegate: 'static + Sized {
    fn set_property_string(
        &mut self,
        key: String,
        value: String,
    ) -> Result<ActionRunResult, ActionRunError>;
    fn set_property_string_any(
        &mut self,
        key: String,
        value: Box<dyn Any>,
    ) -> Result<ActionRunResult, ActionRunError>;
    fn get_property_string(&self, key: String) -> Result<String, ActionRunError>;
}

impl<T: ObjectSetPropertyDelegate> ObjectSetPropertyStringDelegate for T {
    fn set_property_string(
        &mut self,
        key: String,
        value: String,
    ) -> Result<ActionRunResult, ActionRunError> {
        let key = key
            .parse()
            .map_err(|_| ActionRunError::ObjectError(ObjectError::ObjectSetKeyInvalid(key)))?;

        self.set_property(key, value)
    }

    fn set_property_string_any(
        &mut self,
        key: String,
        value: Box<dyn Any>,
    ) -> Result<ActionRunResult, ActionRunError> {
        let key = key
            .parse()
            .map_err(|_| ActionRunError::ObjectError(ObjectError::ObjectSetKeyInvalid(key)))?;

        self.set_property_any(key, value)
    }

    fn get_property_string(&self, key: String) -> Result<String, ActionRunError> {
        let key = key
            .parse()
            .map_err(|_| ActionRunError::ObjectError(ObjectError::ObjectSetKeyInvalid(key)))?;

        self.get_property(key)
    }
}

pub trait ObjectDelegate: 'static + Sized + Display {
    fn default_action(self) -> Option<Action>;
    fn set(
        self,
        preset_handler: &mut PresetHandler,
        updatable_handler: &mut UpdatableHandler,
        fixture_selector_context: FixtureSelectorContext,
        event_list: &mut DemexEventList,
        key: String,
        value: String,
    ) -> Result<ActionRunResult, ActionRunError>;
    fn set_any(
        self,
        preset_handler: &mut PresetHandler,
        updatable_handler: &mut UpdatableHandler,
        fixture_selector_context: FixtureSelectorContext,
        event_list: &mut DemexEventList,
        key: String,
        value: Box<dyn Any>,
    ) -> Result<ActionRunResult, ActionRunError>;
    fn get(
        self,
        preset_handler: &PresetHandler,
        updatable_handler: &UpdatableHandler,
        fixture_selector_context: FixtureSelectorContext,
        key: String,
    ) -> Result<String, ActionRunError>;

    fn get_pool_type_and_id(self) -> Option<(PoolType, u32)>;
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HomeableObject {
    CurrentFixtureSelection,
    FixtureSelector(FixtureSelector),
    Group(u32),
    Executor(u32),
    Programmer,
}

impl HomeableObject {
    fn home_fixture_selection(
        selection: &FixtureSelection,
        fixture_state_handler: &mut FixtureStateHandler,
    ) -> Result<(), ActionRunError> {
        for fixture_path in selection.fixtures() {
            if let Ok(fixture_state) = fixture_state_handler.fixture_mut(fixture_path) {
                // TODO: should we clear the source list here??
                fixture_state
                    .home(false)
                    .map_err(ActionRunError::FixtureError)?;
            }
        }

        Ok(())
    }

    pub fn home(
        &self,
        preset_handler: &PresetHandler,
        fixture_state_handler: &mut FixtureStateHandler,
        updatable_handler: &mut UpdatableHandler,
        fixture_selector_context: FixtureSelectorContext,
        event_list: &mut DemexEventList,
    ) -> Result<ActionRunResult, ActionRunError> {
        match self {
            HomeableObject::CurrentFixtureSelection => {
                if let Some(selection) = fixture_selector_context.current_fixture() {
                    Self::home_fixture_selection(selection, fixture_state_handler)?;
                }
                Ok(ActionRunResult::new())
            }
            HomeableObject::FixtureSelector(fixture_selector) => {
                let selection = fixture_selector
                    .get_selection(preset_handler, fixture_selector_context)
                    .map_err(ActionRunError::FixtureSelectorError)?;

                Self::home_fixture_selection(&selection, fixture_state_handler)?;

                Ok(ActionRunResult::new())
            }
            HomeableObject::Group(group_id) => {
                let group = preset_handler
                    .get_group(*group_id)
                    .map_err(ActionRunError::PresetHandlerError)?;

                Self::home_fixture_selection(group.fixture_selection(), fixture_state_handler)?;

                Ok(ActionRunResult::Default)
            }
            HomeableObject::Executor(executor_id) => {
                if let Ok(fader) = updatable_handler.executor_mut(*executor_id) {
                    fader.stop(fixture_state_handler, preset_handler, event_list);
                }

                Ok(ActionRunResult::new())
            }
            HomeableObject::Programmer => fixture_state_handler
                .home_all(false)
                .map_err(ActionRunError::FixtureError)
                .map(|_| ActionRunResult::new()),
        }
    }
}

impl Display for HomeableObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CurrentFixtureSelection => write!(f, "~"),
            Self::Executor(id) => write!(f, "Executor {}", id),
            Self::Group(id) => write!(f, "Group {}", id),
            Self::FixtureSelector(selector) => write!(f, "{}", selector),
            Self::Programmer => write!(f, "Programmer"),
        }
    }
}

impl ObjectDelegate for HomeableObject {
    fn default_action(self) -> Option<Action> {
        match self {
            Self::FixtureSelector(fixture_selector) => {
                Some(Action::FixtureSelector(fixture_selector))
            }
            _ => Some(Action::Edit(Object::HomeableObject(self))),
        }
    }

    fn get_pool_type_and_id(self) -> Option<(PoolType, u32)> {
        match self {
            Self::Executor(id) => Some((PoolType::Executor, id)),
            Self::Group(id) => Some((PoolType::Group, id)),
            _ => None,
        }
    }

    fn get(
        self,
        preset_handler: &PresetHandler,
        updatable_handler: &UpdatableHandler,
        fixture_selector_context: FixtureSelectorContext,
        key: String,
    ) -> Result<String, ActionRunError> {
        match self {
            Self::Executor(id) => updatable_handler
                .executor(id)
                .map_err(ActionRunError::UpdatableHandlerError)
                .and_then(|executor| executor.get_property_string(key)),
            Self::CurrentFixtureSelection => {
                if let Some(selection) = fixture_selector_context.current_fixture() {
                    selection.get_property_string(key)
                } else {
                    Err(ActionRunError::ObjectError(ObjectError::ObjectNotPresent))
                }
            }
            Self::Group(id) => preset_handler
                .get_group(id)
                .map_err(ActionRunError::PresetHandlerError)
                .and_then(|group| group.get_property_string(key)),
            Self::FixtureSelector(selector) => {
                if let Some(group_id) = selector.try_as_group_id() {
                    HomeableObject::Group(group_id).get(
                        preset_handler,
                        updatable_handler,
                        fixture_selector_context,
                        key,
                    )
                } else {
                    Err(ActionRunError::ActionNotImplementedForObject(
                        "get".to_string(),
                        Object::HomeableObject(Self::FixtureSelector(selector)),
                    ))
                }
            }
            unmatched => Err(ActionRunError::ActionNotImplementedForObject(
                "get".to_string(),
                Object::HomeableObject(unmatched),
            )),
        }
    }

    fn set(
        self,
        preset_handler: &mut PresetHandler,
        updatable_handler: &mut UpdatableHandler,
        fixture_selector_context: FixtureSelectorContext,
        event_list: &mut DemexEventList,
        key: String,
        value: String,
    ) -> Result<ActionRunResult, ActionRunError> {
        match self {
            Self::Executor(executor_id) => updatable_handler
                .executor_mut(executor_id)
                .map_err(ActionRunError::UpdatableHandlerError)
                .and_then(|executor| executor.set_property_string(key, value)),
            Self::CurrentFixtureSelection => {
                if let Some(selection) = fixture_selector_context.current_fixture() {
                    let mut selection = selection.clone();
                    selection.set_property_string(key, value)?;
                    Ok(ActionRunResult::UpdateFixtureSelection(Some(
                        selection.into(),
                    )))
                } else {
                    Ok(ActionRunResult::new())
                }
            }
            Self::Group(id) => preset_handler
                .get_group_mut(id)
                .map_err(ActionRunError::PresetHandlerError)
                .and_then(|group| group.set_property_string(key, value)),
            Self::FixtureSelector(selector) => {
                if let Some(group_id) = selector.try_as_group_id() {
                    HomeableObject::Group(group_id).set(
                        preset_handler,
                        updatable_handler,
                        fixture_selector_context,
                        event_list,
                        key,
                        value,
                    )
                } else {
                    Err(ActionRunError::ActionNotImplementedForObject(
                        "set".to_string(),
                        Object::HomeableObject(Self::FixtureSelector(selector)),
                    ))
                }
            }
            unmatched => Err(ActionRunError::ActionNotImplementedForObject(
                "set".to_string(),
                Object::HomeableObject(unmatched),
            )),
        }
    }

    fn set_any(
        self,
        preset_handler: &mut PresetHandler,
        updatable_handler: &mut UpdatableHandler,
        fixture_selector_context: FixtureSelectorContext,
        event_list: &mut DemexEventList,
        key: String,
        value: Box<dyn Any>,
    ) -> Result<ActionRunResult, ActionRunError> {
        match self {
            Self::Executor(executor_id) => updatable_handler
                .executor_mut(executor_id)
                .map_err(ActionRunError::UpdatableHandlerError)
                .and_then(|executor| executor.set_property_string_any(key, value)),
            Self::CurrentFixtureSelection => {
                if let Some(selection) = fixture_selector_context.current_fixture() {
                    let mut selection = selection.clone();
                    selection.set_property_string_any(key, value)?;
                    Ok(ActionRunResult::UpdateFixtureSelection(Some(
                        selection.into(),
                    )))
                } else {
                    Ok(ActionRunResult::new())
                }
            }
            Self::Group(id) => preset_handler
                .get_group_mut(id)
                .map_err(ActionRunError::PresetHandlerError)
                .and_then(|group| group.set_property_string_any(key, value)),
            Self::FixtureSelector(selector) => {
                if let Some(group_id) = selector.try_as_group_id() {
                    HomeableObject::Group(group_id).set_any(
                        preset_handler,
                        updatable_handler,
                        fixture_selector_context,
                        event_list,
                        key,
                        value,
                    )
                } else {
                    Err(ActionRunError::ActionNotImplementedForObject(
                        "set".to_string(),
                        Object::HomeableObject(Self::FixtureSelector(selector)),
                    ))
                }
            }
            unmatched => Err(ActionRunError::ActionNotImplementedForObject(
                "set".to_string(),
                Object::HomeableObject(unmatched),
            )),
        }
    }
}

impl HomeableObject {
    pub fn rangable_with(&self, other: &HomeableObject) -> bool {
        #[allow(clippy::match_like_matches_macro)]
        match (self, other) {
            (Self::FixtureSelector(_), Self::FixtureSelector(_)) => true,
            (Self::Executor(_), Self::Executor(_)) => true,
            _ => false,
        }
    }
}

impl From<HomeableObject> for Object {
    fn from(value: HomeableObject) -> Self {
        Object::HomeableObject(value)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Object {
    HomeableObject(HomeableObject),
    Sequence(u32),
    SequenceCue(u32, CueIdx),
    ExecutorCue(u32, CueIdx),
    Preset(FixturePresetId),
    Macro(u32),
}

impl Object {
    pub fn cue(seq: u32, cue_idx: CueIdx) -> Self {
        Self::SequenceCue(seq, cue_idx)
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::HomeableObject(object) => object.fmt(f),
            Self::Macro(id) => write!(f, "Macro {}", id),
            Self::Preset(id) => write!(f, "{} Preset {}", id.feature_group, id.preset_id),
            Self::Sequence(id) => write!(f, "Sequence {}", id),
            Self::SequenceCue(id, cue_idx) => {
                write!(f, "Sequence {} Cue {}", id, cue_idx)
            }
            Self::ExecutorCue(id, cue_idx) => {
                write!(f, "Executor {} Cue {}", id, cue_idx)
            }
        }
    }
}

impl ObjectDelegate for Object {
    fn default_action(self) -> Option<Action> {
        match self {
            Self::HomeableObject(homeable_object) => homeable_object.default_action(),
            _ => Some(Action::Edit(self)),
        }
    }

    fn get_pool_type_and_id(self) -> Option<(PoolType, u32)> {
        match self {
            Self::HomeableObject(obj) => obj.get_pool_type_and_id(),
            Self::Macro(id) => Some((PoolType::Macro, id)),
            Self::Sequence(id) => Some((PoolType::Sequence, id)),
            Self::SequenceCue(seq_id, cue_id) => {
                Some((PoolType::SequenceCue(seq_id), cue_id.into()))
            }
            Self::ExecutorCue(_, _) => None,
            Self::Preset(id) => Some((PoolType::Preset(id.feature_group), id.preset_id)),
        }
    }

    fn get(
        self,
        preset_handler: &PresetHandler,
        updatable_handler: &UpdatableHandler,
        fixture_selector_context: FixtureSelectorContext,
        key: String,
    ) -> Result<String, ActionRunError> {
        match self {
            Self::HomeableObject(object) => object.get(
                preset_handler,
                updatable_handler,
                fixture_selector_context,
                key,
            ),
            Self::Macro(macro_id) => preset_handler
                .get_macro(macro_id)
                .map_err(ActionRunError::PresetHandlerError)
                .and_then(|m| m.get_property_string(key)),
            Self::Preset(preset_id) => preset_handler
                .get_preset(preset_id)
                .map_err(ActionRunError::PresetHandlerError)
                .and_then(|preset| preset.get_property_string(key)),
            Self::Sequence(sequence_id) => preset_handler
                .get_sequence(sequence_id)
                .map_err(ActionRunError::PresetHandlerError)
                .and_then(|s| s.get_property_string(key)),
            Self::SequenceCue(sequence_id, cue_idx) => preset_handler
                .get_sequence(sequence_id)
                .and_then(|s| {
                    s.find_cue(cue_idx)
                        .ok_or(PresetHandlerError::CueNotFound(sequence_id, cue_idx))
                })
                .map_err(ActionRunError::PresetHandlerError)
                .and_then(|cue| cue.get_property_string(key)),
            Self::ExecutorCue(executor_id, cue_idx) => updatable_handler
                .executor(executor_id)
                .map_err(ActionRunError::UpdatableHandlerError)
                .and_then(|executor| {
                    let sequence_id = executor.runtime().sequence_id();

                    preset_handler
                        .get_sequence(sequence_id)
                        .and_then(|s| {
                            s.find_cue(cue_idx)
                                .ok_or(PresetHandlerError::CueNotFound(sequence_id, cue_idx))
                        })
                        .map_err(ActionRunError::PresetHandlerError)
                        .and_then(|cue| cue.get_property_string(key))
                }),
        }
    }

    fn set(
        self,
        preset_handler: &mut PresetHandler,
        updatable_handler: &mut UpdatableHandler,
        fixture_selector_context: FixtureSelectorContext,
        event_list: &mut DemexEventList,
        key: String,
        value: String,
    ) -> Result<ActionRunResult, ActionRunError> {
        let cloned_key = key.clone();

        let result = match self.clone() {
            Self::HomeableObject(object) => object.set(
                preset_handler,
                updatable_handler,
                fixture_selector_context,
                event_list,
                key,
                value,
            ),
            Self::Macro(macro_id) => preset_handler
                .get_macro_mut(macro_id)
                .map_err(ActionRunError::PresetHandlerError)
                .and_then(|m| m.set_property_string(key, value)),
            Self::Preset(preset_id) => preset_handler
                .get_preset_mut(preset_id)
                .map_err(ActionRunError::PresetHandlerError)
                .and_then(|preset| preset.set_property_string(key, value)),
            Self::Sequence(sequence_id) => preset_handler
                .get_sequence_mut(sequence_id)
                .map_err(ActionRunError::PresetHandlerError)
                .and_then(|s| s.set_property_string(key, value)),
            Self::SequenceCue(sequence_id, cue_idx) => preset_handler
                .get_sequence_mut(sequence_id)
                .and_then(|s| {
                    s.find_cue_mut(cue_idx)
                        .ok_or(PresetHandlerError::CueNotFound(sequence_id, cue_idx))
                })
                .map_err(ActionRunError::PresetHandlerError)
                .and_then(|cue| cue.set_property_string(key, value)),
            Self::ExecutorCue(executor_id, cue_idx) => updatable_handler
                .executor(executor_id)
                .map_err(ActionRunError::UpdatableHandlerError)
                .and_then(|executor| {
                    let sequence_id = executor.runtime().sequence_id();

                    preset_handler
                        .get_sequence_mut(sequence_id)
                        .and_then(|s| {
                            s.find_cue_mut(cue_idx)
                                .ok_or(PresetHandlerError::CueNotFound(sequence_id, cue_idx))
                        })
                        .map_err(ActionRunError::PresetHandlerError)
                        .and_then(|cue| cue.set_property_string(key, value))
                }),
        };

        event_list.push(DemexEvent::ObjectPropertyChanged(self.clone(), cloned_key));
        result
    }

    fn set_any(
        self,
        preset_handler: &mut PresetHandler,
        updatable_handler: &mut UpdatableHandler,
        fixture_selector_context: FixtureSelectorContext,
        event_list: &mut DemexEventList,
        key: String,
        value: Box<dyn Any>,
    ) -> Result<ActionRunResult, ActionRunError> {
        let cloned_key = key.clone();

        let result = match self.clone() {
            Self::HomeableObject(object) => object.set_any(
                preset_handler,
                updatable_handler,
                fixture_selector_context,
                event_list,
                key,
                value,
            ),
            Self::Macro(macro_id) => preset_handler
                .get_macro_mut(macro_id)
                .map_err(ActionRunError::PresetHandlerError)
                .and_then(|m| m.set_property_string_any(key, value)),
            Self::Preset(preset_id) => preset_handler
                .get_preset_mut(preset_id)
                .map_err(ActionRunError::PresetHandlerError)
                .and_then(|preset| preset.set_property_string_any(key, value)),
            Self::Sequence(sequence_id) => preset_handler
                .get_sequence_mut(sequence_id)
                .map_err(ActionRunError::PresetHandlerError)
                .and_then(|s| s.set_property_string_any(key, value)),
            Self::SequenceCue(sequence_id, cue_idx) => preset_handler
                .get_sequence_mut(sequence_id)
                .and_then(|s| {
                    s.find_cue_mut(cue_idx)
                        .ok_or(PresetHandlerError::CueNotFound(sequence_id, cue_idx))
                })
                .map_err(ActionRunError::PresetHandlerError)
                .and_then(|cue| cue.set_property_string_any(key, value)),
            Self::ExecutorCue(executor_id, cue_idx) => updatable_handler
                .executor(executor_id)
                .map_err(ActionRunError::UpdatableHandlerError)
                .and_then(|executor| {
                    let sequence_id = executor.runtime().sequence_id();

                    preset_handler
                        .get_sequence_mut(sequence_id)
                        .and_then(|s| {
                            s.find_cue_mut(cue_idx)
                                .ok_or(PresetHandlerError::CueNotFound(sequence_id, cue_idx))
                        })
                        .map_err(ActionRunError::PresetHandlerError)
                        .and_then(|cue| cue.set_property_string_any(key, value))
                }),
        };

        event_list.push(DemexEvent::ObjectPropertyChanged(self.clone(), cloned_key));
        result
    }
}

impl Object {
    pub fn rangable_with(&self, other: &Object) -> bool {
        match (self, other) {
            (
                Self::HomeableObject(homeable_object),
                Self::HomeableObject(other_homeable_object),
            ) => homeable_object.rangable_with(other_homeable_object),
            (Self::Sequence(_), Self::Sequence(_)) => true,
            (Self::SequenceCue(sequence_id_a, _), Self::SequenceCue(sequence_id_b, _)) => {
                sequence_id_a == sequence_id_b
            }
            (Self::Preset(_), Self::Preset(_)) => true,
            (Self::Macro(_), Self::Macro(_)) => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectRange {
    from: Object,
    to: Object,
}

impl ObjectRange {
    pub fn new(from: Object, to: Object) -> Result<Self, ObjectError> {
        if from.rangable_with(&to) {
            Ok(Self { from, to })
        } else {
            Err(ObjectError::ObjectVariantMismatch(from, to))
        }
    }

    pub fn single(object: Object) -> Self {
        Self {
            from: object.clone(),
            to: object,
        }
    }

    pub fn from(&self) -> &Object {
        &self.from
    }

    pub fn to(&self) -> &Object {
        &self.to
    }

    pub fn is_single(&self) -> bool {
        self.from == self.to
    }

    pub fn as_tuple(&self) -> (&Object, &Object) {
        (&self.from, &self.to)
    }
}
