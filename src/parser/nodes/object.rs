use serde::{Deserialize, Serialize};

use super::{action::Action, fixture_selector::FixtureSelector};

pub trait ObjectTrait {
    fn default_action(self) -> Option<Action>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HomeableObject {
    FixtureSelector(FixtureSelector),
}

impl ObjectTrait for HomeableObject {
    fn default_action(self) -> Option<Action> {
        match self {
            Self::FixtureSelector(fixture_selector) => {
                Some(Action::FixtureSelector(fixture_selector))
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Object {
    HomeableObject(HomeableObject),
}

impl ObjectTrait for Object {
    fn default_action(self) -> Option<Action> {
        match self {
            Self::HomeableObject(homeable_object) => homeable_object.default_action(),
        }
    }
}
