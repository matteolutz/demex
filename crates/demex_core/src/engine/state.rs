use std::{collections::HashMap, sync::Arc};

use crate::{
    engine::component::Component, patch::Patch, selection::FixtureSelection,
    state::fixture_state::FixtureState,
};

#[derive(Debug, Clone, Default)]
pub struct DemexEngineState {
    pub fixture_selection: Option<FixtureSelection>,
}

impl Component for DemexEngineState {}

pub struct DemexFrontendInitState {
    pub fixture_selection: Option<FixtureSelection>,
    pub fixture_states: HashMap<u32, FixtureState>,
    pub patch: Arc<Patch>,
}
