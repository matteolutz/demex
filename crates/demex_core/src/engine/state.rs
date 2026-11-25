use std::collections::HashMap;

use crate::{
    channel3::attribute::FixtureChannel3Attribute,
    engine::component::Component,
    fixture::FixturePath,
    patch::Patch,
    pool::{PoolItem, PoolType},
    selection::FixtureSelection,
    state::fixture_state::FixtureState,
    timing::speed_master::SpeedMasterValue,
};

#[derive(Debug, Clone, Default)]
pub struct DemexEngineState {
    /// The currently selected fixtures.
    pub fixture_selection: Option<FixtureSelection>,

    /// The encoder attributes that are visible to the frontend.
    /// This information is used to have the same encoders on input devices.
    pub visible_encoder_attributes: Vec<FixtureChannel3Attribute>,

    /// The currently highlighted fixtures.
    pub highlight: Option<FixtureSelection>,
}

impl Component for DemexEngineState {}

#[derive(Debug)]
pub struct DemexFrontendInitState {
    pub fixture_selection: Option<FixtureSelection>,
    pub fixture_states: HashMap<FixturePath, FixtureState>,
    pub pools: HashMap<PoolType, Vec<PoolItem>>,
    pub speedmasters: HashMap<u32, SpeedMasterValue>,
    pub patch: Patch,
}
