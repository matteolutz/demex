use crate::{engine::component::Component, selection::FixtureSelection};

#[derive(Debug, Clone, Default)]
pub struct DemexEngineState {
    pub fixture_selection: Option<FixtureSelection>,
}

impl Component for DemexEngineState {}
