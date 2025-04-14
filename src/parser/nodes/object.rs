use serde::{Deserialize, Serialize};

use crate::{
    fixture::{
        handler::FixtureHandler,
        presets::{preset::FixturePresetId, PresetHandler},
        sequence::cue::CueIdx,
        updatables::UpdatableHandler,
    },
    ui::window::edit::DemexEditWindow,
};

use super::{
    action::{error::ActionRunError, result::ActionRunResult, Action},
    fixture_selector::{FixtureSelector, FixtureSelectorContext},
};

#[derive(Debug)]
pub enum ObjectError {
    ObjectVariantMismatch(Object, Object),
}

impl std::fmt::Display for ObjectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ObjectError::ObjectVariantMismatch(from, to) => {
                write!(f, "Object variant mismatch: {:?} != {:?}", from, to)
            }
        }
    }
}

impl std::error::Error for ObjectError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

pub trait ObjectTrait {
    fn default_action(self) -> Option<Action>;
    fn edit_window(self) -> Option<DemexEditWindow>;
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HomeableObject {
    FixtureSelector(FixtureSelector),
    Executor(u32),
    Fader(u32),
    Programmer,
}

impl HomeableObject {
    pub fn run_home(
        &self,
        preset_handler: &PresetHandler,
        fixture_handler: &mut FixtureHandler,
        updatable_handler: &mut UpdatableHandler,
        fixture_selector_context: FixtureSelectorContext,
    ) -> Result<ActionRunResult, ActionRunError> {
        match self {
            HomeableObject::FixtureSelector(fixture_selector) => {
                let selection = fixture_selector
                    .get_selection(preset_handler, fixture_selector_context)
                    .map_err(ActionRunError::FixtureSelectorError)?;

                for fixture_id in selection.fixtures() {
                    if let Some(fixture) = fixture_handler.fixture(*fixture_id) {
                        // TODO: should we clear the source list here??
                        fixture.home(false).map_err(ActionRunError::FixtureError)?;
                    }
                }

                Ok(ActionRunResult::new())
            }
            HomeableObject::Executor(executor_id) => {
                updatable_handler
                    .stop_executor(*executor_id, fixture_handler, preset_handler)
                    .map_err(ActionRunError::UpdatableHandlerError)?;

                Ok(ActionRunResult::new())
            }
            HomeableObject::Fader(fader_id) => {
                if let Ok(fader) = updatable_handler.fader_mut(*fader_id) {
                    fader.home(fixture_handler, preset_handler);
                }

                Ok(ActionRunResult::new())
            }
            HomeableObject::Programmer => fixture_handler
                .home_all(false)
                .map_err(ActionRunError::FixtureHandlerError)
                .map(|_| ActionRunResult::new()),
        }
    }
}

impl ObjectTrait for HomeableObject {
    fn default_action(self) -> Option<Action> {
        match self {
            Self::FixtureSelector(fixture_selector) => {
                Some(Action::FixtureSelector(fixture_selector))
            }
            _ => Some(Action::Edit(Object::HomeableObject(self))),
        }
    }

    fn edit_window(self) -> Option<DemexEditWindow> {
        match self {
            Self::Executor(id) => Some(DemexEditWindow::EditExecutor(id)),
            Self::Fader(id) => Some(DemexEditWindow::EditFader(id)),
            Self::FixtureSelector(fixture_selector) => fixture_selector
                .try_as_group_id()
                .map(DemexEditWindow::EditGroup),
            Self::Programmer => None,
        }
    }
}

impl HomeableObject {
    pub fn rangable_with(&self, other: &HomeableObject) -> bool {
        match (self, other) {
            (Self::FixtureSelector(_), Self::FixtureSelector(_)) => true,
            (Self::Executor(_), Self::Executor(_)) => true,
            (Self::Fader(_), Self::Fader(_)) => true,
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

impl ObjectTrait for Object {
    fn default_action(self) -> Option<Action> {
        match self {
            Self::HomeableObject(homeable_object) => homeable_object.default_action(),
            _ => Some(Action::Edit(self)),
        }
    }

    fn edit_window(self) -> Option<DemexEditWindow> {
        match self {
            Self::HomeableObject(obj) => obj.edit_window(),
            Self::Sequence(id) => Some(DemexEditWindow::EditSequence(id)),
            Self::SequenceCue(sequence_id, cue_idx) => {
                Some(DemexEditWindow::EditSequenceCue(sequence_id, cue_idx))
            }
            Self::Preset(preset_id) => Some(DemexEditWindow::EditPreset(preset_id)),
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
