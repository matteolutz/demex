use serde::{Deserialize, Serialize};

use crate::{
    fixture::{handler::FixtureHandler, updatables::UpdatableHandler},
    parser::nodes::{
        action::{error::ActionRunError, result::ActionRunResult, Action},
        fixture_selector::FixtureSelectorContext,
    },
};

use super::PresetHandler;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MMacro {
    id: u32,
    name: String,
    action: Box<Action>,
}

impl MMacro {
    pub fn new(id: u32, name: Option<String>, action: Box<Action>) -> Self {
        MMacro {
            id,
            name: name.unwrap_or(format!("Macro {}", id)),
            action,
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn name_mut(&mut self) -> &mut String {
        &mut self.name
    }

    pub fn action(&self) -> &Action {
        &self.action
    }

    pub fn run(
        &self,
        fixture_handler: &mut FixtureHandler,
        preset_handler: &mut PresetHandler,
        fixture_selector_context: FixtureSelectorContext,
        update_handler: &mut UpdatableHandler,
    ) -> Result<ActionRunResult, ActionRunError> {
        self.action.run(
            fixture_handler,
            preset_handler,
            fixture_selector_context,
            update_handler,
        )
    }
}
