use egui_probe::EguiProbe;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, EguiProbe)]
pub struct FixtureSelection {
    fixtures: Vec<u32>,

    group_size: usize,
    #[serde(default)]
    wings: usize,
}

impl Default for FixtureSelection {
    fn default() -> Self {
        Self {
            fixtures: Vec::new(),
            group_size: 1,
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

    pub fn fixtures(&self) -> &[u32] {
        &self.fixtures
    }

    pub fn wings(&self) -> usize {
        self.wings.max(1)
    }

    pub fn group_size(&self) -> usize {
        self.group_size.max(1)
    }

    pub fn num_groups(&self) -> usize {
        self.fixtures.len() / self.group_size
    }

    pub fn offset_idx(&self, fixture_id: u32) -> Option<usize> {
        let fixture_position = self.fixtures.iter().position(|&id| id == fixture_id)?;

        let grouped_offset = fixture_position / self.group_size();

        Some(grouped_offset % (self.num_groups() / self.wings()))
    }

    pub fn num_offsets(&self) -> usize {
        let num_groups = self.num_groups();
        num_groups / self.wings()
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
