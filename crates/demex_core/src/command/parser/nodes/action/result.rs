use crate::{
    event::FixtureSelectionWithGroup,
    input::control::{DemexInputDeviceControlAssignment, DemexInputDeviceControlUnassignment},
    patch::Patch,
};

#[derive(Debug, Clone, Default)]
pub enum ActionRunResult {
    #[default]
    Default,

    Info(String),
    InfoWithLink(String, String),
    Warn(String),

    UpdateFixtureSelection(Option<FixtureSelectionWithGroup>),
    UpdateHighlight(Option<FixtureSelectionWithGroup>),
    UpdatePatch(Patch),

    Assign(DemexInputDeviceControlAssignment),
    AssignMultiple(Vec<DemexInputDeviceControlAssignment>),
    Unassign(DemexInputDeviceControlUnassignment),

    GroupFixturesChanges(u32),
    GroupAdded(u32),
    GroupsRemoved(Vec<u32>),

    Lock,

    Save,
}

impl ActionRunResult {
    pub fn new() -> Self {
        Self::Default
    }
}
