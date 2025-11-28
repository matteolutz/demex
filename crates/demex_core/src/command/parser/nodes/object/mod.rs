use serde::{Deserialize, Serialize};

use crate::{
    event::DemexEvent,
    presets::{PresetHandler, error::PresetHandlerError, preset::FixturePresetId},
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

                    // FIXME: send event
                    Ok(ActionRunResult::new())
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
}

trait ObjectSetPropertyStringDelegate: 'static + Sized {
    fn set_property_string(
        &mut self,
        key: String,
        value: String,
    ) -> Result<ActionRunResult, ActionRunError>;
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
}

pub trait ObjectDelegate: 'static + Sized {
    fn default_action(self) -> Option<Action>;
    fn set(
        self,
        preset_handler: &mut PresetHandler,
        updatable_handler: &mut UpdatableHandler,
        key: String,
        value: String,
    ) -> Result<ActionRunResult, ActionRunError>;

    #[cfg(feature = "ui")]
    fn edit_window(self) -> Option<crate::ui::window::edit::DemexEditWindow>;
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HomeableObject {
    FixtureSelector(FixtureSelector),
    Executor(u32),
    Programmer,
}

impl HomeableObject {
    pub fn home(
        &self,
        preset_handler: &PresetHandler,
        fixture_state_handler: &mut FixtureStateHandler,
        updatable_handler: &mut UpdatableHandler,
        fixture_selector_context: FixtureSelectorContext,
    ) -> Result<ActionRunResult, ActionRunError> {
        match self {
            HomeableObject::FixtureSelector(fixture_selector) => {
                let selection = fixture_selector
                    .get_selection(preset_handler, fixture_selector_context)
                    .map_err(ActionRunError::FixtureSelectorError)?;

                for fixture_id in selection.fixtures() {
                    if let Ok(fixture_state) = fixture_state_handler.fixture_mut(*fixture_id) {
                        // TODO: should we clear the source list here??
                        fixture_state
                            .home(false)
                            .map_err(ActionRunError::FixtureError)?;
                    }
                }

                Ok(ActionRunResult::new())
            }
            HomeableObject::Executor(executor_id) => {
                if let Ok(fader) = updatable_handler.executor_mut(*executor_id) {
                    fader.stop(fixture_state_handler, preset_handler);
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

impl ObjectDelegate for HomeableObject {
    fn default_action(self) -> Option<Action> {
        match self {
            Self::FixtureSelector(fixture_selector) => {
                Some(Action::FixtureSelector(fixture_selector))
            }
            _ => Some(Action::Edit(Object::HomeableObject(self))),
        }
    }

    fn set(
        self,
        _preset_handler: &mut PresetHandler,
        updatable_handler: &mut UpdatableHandler,
        key: String,
        value: String,
    ) -> Result<ActionRunResult, ActionRunError> {
        match self {
            Self::Executor(executor_id) => updatable_handler
                .executor_mut(executor_id)
                .map_err(ActionRunError::UpdatableHandlerError)
                .and_then(|executor| executor.set_property_string(key, value)),
            unmatched => Err(ActionRunError::ActionNotImplementedForObject(
                "set".to_string(),
                Object::HomeableObject(unmatched),
            )),
        }
    }

    #[cfg(feature = "ui")]
    fn edit_window(self) -> Option<crate::ui::window::edit::DemexEditWindow> {
        match self {
            Self::Executor(id) => Some(crate::ui::window::edit::DemexEditWindow::EditExecutor(id)),
            Self::FixtureSelector(fixture_selector) => fixture_selector
                .try_as_group_id()
                .map(crate::ui::window::edit::DemexEditWindow::EditGroup),
            Self::Programmer => None,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Object {
    HomeableObject(HomeableObject),
    Sequence(u32),
    SequenceCue(u32, CueIdx),
    Preset(FixturePresetId),
    Macro(u32),
}

impl ObjectDelegate for Object {
    fn default_action(self) -> Option<Action> {
        match self {
            Self::HomeableObject(homeable_object) => homeable_object.default_action(),
            _ => Some(Action::Edit(self)),
        }
    }

    fn set(
        self,
        preset_handler: &mut PresetHandler,
        updatable_handler: &mut UpdatableHandler,
        key: String,
        value: String,
    ) -> Result<ActionRunResult, ActionRunError> {
        let cloned_key = key.clone();

        let result = match self.clone() {
            Self::HomeableObject(object) => {
                object.set(preset_handler, updatable_handler, key, value)
            }
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
        };

        result.map(|result| ActionRunResult::WithEvent {
            result: Box::new(result),
            event: DemexEvent::ObjectPropertyChanged(self.clone(), cloned_key),
        })
    }

    #[cfg(feature = "ui")]
    fn edit_window(self) -> Option<crate::ui::window::edit::DemexEditWindow> {
        match self {
            Self::HomeableObject(obj) => obj.edit_window(),
            Self::Sequence(id) => Some(crate::ui::window::edit::DemexEditWindow::EditSequence(id)),
            Self::SequenceCue(sequence_id, cue_idx) => Some(
                crate::ui::window::edit::DemexEditWindow::EditSequenceCue(sequence_id, cue_idx),
            ),
            Self::Preset(preset_id) => Some(crate::ui::window::edit::DemexEditWindow::EditPreset(
                preset_id,
            )),
            Self::Macro(_) => None,
        }
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
