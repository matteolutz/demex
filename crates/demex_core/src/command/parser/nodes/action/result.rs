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

    Lock,

    Save,
}

impl ActionRunResult {
    pub fn new() -> Self {
        Self::Default
    }
}
