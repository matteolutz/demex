use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::{
    channel3::attribute::FixtureChannel3Attribute,
    command::parser::nodes::fixture_selector::{FixtureSelector, FixtureSelectorContext},
    fixture::{FixturePath, FixturePathMatchLevel},
    implement_set_property,
    patch::Patch,
};

use super::presets::PresetHandler;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FixtureSelection {
    fixtures: Vec<FixturePath>,

    #[serde(default)]
    group: usize,
    #[serde(default)]
    block: usize,

    wings: usize,

    #[serde(default)]
    reverse: bool,
}

impl Default for FixtureSelection {
    fn default() -> Self {
        Self {
            fixtures: Vec::new(),

            group: 1,
            block: 1,
            wings: 1,
            reverse: false,
        }
    }
}

impl FixtureSelection {
    pub fn get_attributes(&self, patch: &Patch) -> HashSet<FixtureChannel3Attribute> {
        self.fixtures
            .iter()
            .filter_map(|fixture_path| patch.fixture(fixture_path).ok())
            .flat_map(|fixture| fixture.get_attributes_recursive(patch))
            .collect()
    }
}

impl FixtureSelection {
    pub fn has_fixture(&self, fixture_path: &FixturePath) -> bool {
        self.fixtures.contains(fixture_path)
    }

    pub fn has_fixture_with_level(
        &self,
        fixture_path: &FixturePath,
        level: FixturePathMatchLevel,
    ) -> bool {
        self.fixtures
            .iter()
            .any(|path| path.matches(fixture_path, level))
    }

    pub fn intersects_with(&self, other: &FixtureSelection) -> bool {
        self.fixtures.iter().any(|id| other.has_fixture(id))
    }

    pub fn retain(&mut self, f: impl Fn(&FixturePath) -> bool) {
        self.fixtures.retain(f);
    }

    pub fn extend_from(&mut self, other: &FixtureSelection) {
        for fixture in other.fixtures() {
            if self.fixtures.contains(fixture) {
                continue;
            }
            self.fixtures.push(*fixture);
        }
    }

    pub fn update_from(&mut self, other: &FixtureSelection) {
        // merge fixture list
        for fixture in other.fixtures() {
            if self.fixtures.contains(fixture) {
                continue;
            }

            self.fixtures.push(*fixture);
        }

        // update group, block and wings
        self.group = other.group;
        self.block = other.block;
        self.wings = other.wings;
    }

    pub fn subtract(&mut self, other: &FixtureSelection) {
        self.fixtures.retain(|f| !other.has_fixture(f));
    }

    pub fn equals_selector(
        &self,
        selector: &FixtureSelector,
        preset_handler: &PresetHandler,
        context: FixtureSelectorContext,
    ) -> bool {
        let selection = selector.get_selection(preset_handler, context);
        selection.is_ok_and(|selection| &selection == self)
    }

    pub fn with_additional_fixtures(mut self, fixtures: &[FixturePath]) -> Self {
        for fixture in fixtures {
            if self.fixtures.contains(fixture) {
                continue;
            }
            self.fixtures.push(*fixture);
        }

        self
    }

    pub fn master_fixture(&self) -> Option<&FixturePath> {
        self.fixtures.first()
    }

    pub fn fixtures(&self) -> &[FixturePath] {
        &self.fixtures
    }

    pub fn group(&self) -> usize {
        self.group.max(1)
    }

    pub fn group_mut(&mut self) -> &mut usize {
        &mut self.group
    }

    pub fn block(&self) -> usize {
        self.block.max(1)
    }

    pub fn block_mut(&mut self) -> &mut usize {
        &mut self.block
    }

    pub fn wings(&self) -> usize {
        self.wings.max(1)
    }

    pub fn wings_mut(&mut self) -> &mut usize {
        &mut self.wings
    }

    pub fn reverse(&self) -> bool {
        self.reverse
    }

    pub fn reverse_mut(&mut self) -> &mut bool {
        &mut self.reverse
    }

    pub fn offset(&self, fixture_path: &FixturePath) -> Option<f32> {
        Some(self.offset_idx(fixture_path)? as f32 / self.num_offsets() as f32)
    }

    pub fn offset_idx(&self, fixture_path: &FixturePath) -> Option<usize> {
        let fixture_position = self.fixtures.iter().position(|path| path == fixture_path)?;

        let blocked_offset = fixture_position / self.block();

        // let grouped_offset = blocked_offset % (self.num_blocks() / self.group());
        let grouped_offset = if self.group() == 1 {
            blocked_offset
        } else {
            blocked_offset % self.group()
        };

        let wing_size = self.num_grouped_offsets() / self.wings();
        let mut wing_offset = grouped_offset % wing_size.max(1);

        // it's an odd wing
        if (grouped_offset / wing_size) % 2 != 0 {
            wing_offset = wing_size - wing_offset - 1;
        }

        Some(if self.reverse {
            self.num_offsets() - 1 - wing_offset
        } else {
            wing_offset
        })
    }

    fn num_blocked_offsets(&self) -> usize {
        self.fixtures.len().div_ceil(self.block())
    }

    fn num_grouped_offsets(&self) -> usize {
        if self.group() == 1 {
            self.num_blocked_offsets()
        } else {
            self.group()
        }
    }

    pub fn num_offsets(&self) -> usize {
        self.num_grouped_offsets() / self.wings()
    }
}

impl From<Vec<FixturePath>> for FixtureSelection {
    fn from(fixtures: Vec<FixturePath>) -> Self {
        Self {
            fixtures,
            ..Default::default()
        }
    }
}

#[derive(Copy, Clone, strum_macros::EnumString, strum_macros::Display, strum::EnumIter)]
pub enum FixtureSelectionProperty {
    Group,
    Block,
    Wings,
    Reverse,
}

implement_set_property! {
    for FixtureSelection with FixtureSelectionProperty,

    Group => group as usize,
    Block => block as usize,
    Wings => wings as usize,
    Reverse => reverse as bool
}

#[cfg(test)]
mod tests {
    use crate::fixture::{FixtureId, FixturePath};

    use super::FixtureSelection;

    fn even_num_fixtures() -> Vec<FixturePath> {
        (1..=10)
            .map(|id| FixtureId::new(id).unwrap().into())
            .collect()
    }

    fn odd_num_fixtures() -> Vec<FixturePath> {
        (1..=11)
            .map(|id| FixtureId::new(id).unwrap().into())
            .collect()
    }

    fn assert_offsets_equal(selection: &FixtureSelection, offsets: &[usize]) {
        assert_eq!(
            selection
                .fixtures()
                .iter()
                .map(|f| selection.offset_idx(f).unwrap())
                .collect::<Vec<_>>(),
            offsets
        );
    }

    #[test]
    fn test_offset_basic_even() {
        let selection = super::FixtureSelection {
            fixtures: even_num_fixtures(),
            group: 1,
            block: 1,
            wings: 1,
            reverse: false,
        };

        assert_offsets_equal(&selection, &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
        assert_eq!(selection.num_offsets(), 10);
    }

    #[test]
    fn test_offset_basic_odd() {
        let selection = super::FixtureSelection {
            fixtures: odd_num_fixtures(),
            group: 1,
            block: 1,
            wings: 1,
            reverse: false,
        };

        assert_offsets_equal(&selection, &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        assert_eq!(selection.num_offsets(), 11);
    }

    #[test]
    fn test_block_even() {
        let selection = super::FixtureSelection {
            fixtures: even_num_fixtures(),
            group: 1,
            block: 2,
            wings: 1,
            reverse: false,
        };

        assert_offsets_equal(&selection, &[0, 0, 1, 1, 2, 2, 3, 3, 4, 4]);
        assert_eq!(selection.num_offsets(), 5);
    }

    #[test]
    fn test_block_odd() {
        let selection = super::FixtureSelection {
            fixtures: odd_num_fixtures(),
            group: 1,
            block: 2,
            wings: 1,
            reverse: false,
        };

        assert_offsets_equal(&selection, &[0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5]);
        assert_eq!(selection.num_offsets(), 6);
    }

    #[test]
    fn test_group_even() {
        let selection = super::FixtureSelection {
            fixtures: even_num_fixtures(),
            group: 2,
            block: 1,
            wings: 1,
            reverse: false,
        };

        assert_offsets_equal(&selection, &[0, 1, 0, 1, 0, 1, 0, 1, 0, 1]);
        assert_eq!(selection.num_offsets(), 2);
    }

    #[test]
    fn test_group_odd() {
        let selection = super::FixtureSelection {
            fixtures: odd_num_fixtures(),
            group: 2,
            block: 1,
            wings: 1,
            reverse: false,
        };

        assert_offsets_equal(&selection, &[0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0]);
        assert_eq!(selection.num_offsets(), 2);
    }

    #[test]
    fn test_group_even_three() {
        let selection = super::FixtureSelection {
            fixtures: even_num_fixtures(),
            group: 3,
            block: 1,
            wings: 1,
            reverse: false,
        };

        assert_offsets_equal(&selection, &[0, 1, 2, 0, 1, 2, 0, 1, 2, 0]);
        assert_eq!(selection.num_offsets(), 3);
    }

    #[test]
    fn test_group_odd_three() {
        let selection = super::FixtureSelection {
            fixtures: odd_num_fixtures(),
            group: 3,
            block: 1,
            wings: 1,
            reverse: false,
        };

        assert_offsets_equal(&selection, &[0, 1, 2, 0, 1, 2, 0, 1, 2, 0, 1]);
        assert_eq!(selection.num_offsets(), 3);
    }

    #[test]
    fn test_wings_two_even() {
        let selection = super::FixtureSelection {
            fixtures: even_num_fixtures(),
            group: 1,
            block: 1,
            wings: 2,
            reverse: false,
        };

        assert_offsets_equal(&selection, &[0, 1, 2, 3, 4, 4, 3, 2, 1, 0]);
        assert_eq!(selection.num_offsets(), 5);
    }

    #[test]
    fn test_wings_two_odd() {
        let selection = super::FixtureSelection {
            fixtures: odd_num_fixtures(),
            group: 1,
            block: 1,
            wings: 2,
            reverse: false,
        };

        assert_offsets_equal(&selection, &[0, 1, 2, 3, 4, 4, 3, 2, 1, 0, 0]);
        assert_eq!(selection.num_offsets(), 5);
    }

    #[test]
    fn test_wings_three_even() {
        let selection = super::FixtureSelection {
            fixtures: even_num_fixtures(),
            group: 1,
            block: 1,
            wings: 3,
            reverse: false,
        };

        assert_offsets_equal(&selection, &[0, 1, 2, 2, 1, 0, 0, 1, 2, 2]);
        assert_eq!(selection.num_offsets(), 3);
    }

    #[test]
    fn test_wings_three_odd() {
        let selection = super::FixtureSelection {
            fixtures: odd_num_fixtures(),
            group: 1,
            block: 1,
            wings: 3,
            reverse: false,
        };

        assert_offsets_equal(&selection, &[0, 1, 2, 2, 1, 0, 0, 1, 2, 2, 1]);
        assert_eq!(selection.num_offsets(), 3);
    }

    #[test]
    fn test_wings_four_even() {
        let selection = super::FixtureSelection {
            fixtures: even_num_fixtures(),
            group: 1,
            block: 1,
            wings: 4,
            reverse: false,
        };

        assert_offsets_equal(&selection, &[0, 1, 1, 0, 0, 1, 1, 0, 0, 1]);
        assert_eq!(selection.num_offsets(), 2);
    }

    #[test]
    fn test_wings_four_odd() {
        let selection = super::FixtureSelection {
            fixtures: odd_num_fixtures(),
            group: 1,
            block: 1,
            wings: 4,
            reverse: false,
        };

        assert_offsets_equal(&selection, &[0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1]);
        assert_eq!(selection.num_offsets(), 2);
    }
}
