use crate::selection::FixtureSelection;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixtureSelectionWithGroup {
    pub(crate) selection: FixtureSelection,
    pub(crate) group_id: Option<u32>,
}

impl FixtureSelectionWithGroup {
    pub fn with_group(selection: FixtureSelection, group_id: Option<u32>) -> Self {
        Self {
            selection,
            group_id,
        }
    }

    pub fn selection(&self) -> &FixtureSelection {
        &self.selection
    }

    pub fn group_id(&self) -> Option<u32> {
        self.group_id
    }
}

impl From<FixtureSelection> for FixtureSelectionWithGroup {
    fn from(selection: FixtureSelection) -> Self {
        Self {
            selection,
            group_id: None,
        }
    }
}
