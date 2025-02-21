use egui_probe::EguiProbe;
use serde::{Deserialize, Serialize};

use crate::parser::nodes::fixture_selector::{FixtureSelector, FixtureSelectorContext};

use super::presets::PresetHandler;

#[derive(Debug, Clone, Serialize, Deserialize, EguiProbe, PartialEq, Eq)]
pub struct FixtureSelection {
    fixtures: Vec<u32>,

    #[serde(default)]
    group: usize,
    #[serde(default)]
    block: usize,

    wings: usize,
}

impl Default for FixtureSelection {
    fn default() -> Self {
        Self {
            fixtures: Vec::new(),

            group: 1,
            block: 1,
            wings: 1,
        }
    }
}

impl FixtureSelection {
    pub fn has_fixture(&self, fixture_id: u32) -> bool {
        self.fixtures.contains(&fixture_id)
    }

    pub fn intersects_with(&self, other: &FixtureSelection) -> bool {
        self.fixtures.iter().any(|id| other.has_fixture(*id))
    }

    pub fn equals_selector(
        &self,
        selector: &FixtureSelector,
        preset_handler: &PresetHandler,
        context: FixtureSelectorContext,
    ) -> bool {
        let fixtures = selector.get_fixtures(preset_handler, context);
        if let Ok(fixtures) = fixtures {
            &FixtureSelection::from(fixtures) == self
        } else {
            false
        }
    }

    pub fn add_fixtures(&mut self, fixtures: &[u32]) {
        for fixture in fixtures {
            if self.fixtures.contains(fixture) {
                continue;
            }
            self.fixtures.push(*fixture);
        }
    }

    pub fn fixtures(&self) -> &[u32] {
        &self.fixtures
    }

    pub fn group(&self) -> usize {
        self.group.max(1)
    }

    pub fn block(&self) -> usize {
        self.block.max(1)
    }

    pub fn wings(&self) -> usize {
        self.wings.max(1)
    }

    pub fn offset(&self, fixture_id: u32) -> Option<f32> {
        Some(self.offset_idx(fixture_id)? as f32 / self.num_offsets() as f32)
    }

    pub fn offset_idx(&self, fixture_id: u32) -> Option<usize> {
        let fixture_position = self.fixtures.iter().position(|&id| id == fixture_id)?;

        let blocked_offset = fixture_position / self.block();

        // let grouped_offset = blocked_offset % (self.num_blocks() / self.group());
        let grouped_offset = if self.group() == 1 {
            blocked_offset
        } else {
            blocked_offset % self.group()
        };

        let wing_size = self.num_grouped_offsets() / self.wings();
        let mut wing_offset = grouped_offset % wing_size;

        // it's an odd wing
        if (grouped_offset / wing_size) % 2 != 0 {
            wing_offset = wing_size - wing_offset - 1;
        }

        Some(wing_offset)
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

impl From<Vec<u32>> for FixtureSelection {
    fn from(fixtures: Vec<u32>) -> Self {
        Self {
            fixtures,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::FixtureSelection;

    fn even_num_fixtures() -> Vec<u32> {
        (1..=10).collect()
    }

    fn odd_num_fixtures() -> Vec<u32> {
        (1..=11).collect()
    }

    fn assert_offsets_equal(selection: &FixtureSelection, offsets: &[usize]) {
        assert_eq!(
            selection
                .fixtures()
                .into_iter()
                .map(|f| selection.offset_idx(*f).unwrap())
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
        };

        assert_offsets_equal(&selection, &[0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1]);
        assert_eq!(selection.num_offsets(), 2);
    }
}
