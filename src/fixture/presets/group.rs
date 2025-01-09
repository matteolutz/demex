use crate::parser::nodes::fixture_selector::{
    FixtureSelector, FixtureSelectorContext, FixtureSelectorError,
};

use super::PresetHandler;

#[derive(Debug)]
pub struct FixtureGroup {
    id: u32,
    name: String,
    fixture_selector: FixtureSelector,
}

impl FixtureGroup {
    pub fn new(id: u32, fixture_selector: FixtureSelector) -> Self {
        assert!(
            fixture_selector.is_flat(),
            "FixtureGroup fixture selector must be flattened"
        );

        FixtureGroup {
            id,
            name: format!("Group {}", id),
            fixture_selector,
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

    pub fn get_fixtures(
        &self,
        preset_handler: &PresetHandler,
        context: FixtureSelectorContext,
    ) -> Result<Vec<u32>, FixtureSelectorError> {
        self.fixture_selector.get_fixtures(preset_handler, context)
    }
}
