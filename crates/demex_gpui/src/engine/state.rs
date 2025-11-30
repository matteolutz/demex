use std::{collections::HashMap, sync::Arc};

use demex_core::{
    channel3::channel_value::FixtureChannelValue3,
    engine::{state::DemexFrontendInitState, tick::DemexEngineTickState},
    event::DemexEvent,
    patch::Patch,
    selection::FixtureSelection,
};
use gpui::{App, AppContext, Entity, Global};

pub struct DemexUiState {
    fixture_selection: Entity<Option<FixtureSelection>>,
    fixture_values: Entity<HashMap<u32, HashMap<String, FixtureChannelValue3>>>,
    patch: Entity<Arc<Patch>>,
}

impl DemexUiState {
    pub fn fixture_selection(cx: &App) -> Entity<Option<FixtureSelection>> {
        let this: &Self = cx.global();
        this.fixture_selection.clone()
    }

    pub fn fixture_values(cx: &App) -> Entity<HashMap<u32, HashMap<String, FixtureChannelValue3>>> {
        let this: &Self = cx.global();
        this.fixture_values.clone()
    }

    pub fn patch(cx: &App) -> Entity<Arc<Patch>> {
        let this: &Self = cx.global();
        this.patch.clone()
    }
}

impl DemexUiState {
    pub fn new(frontend_state: DemexFrontendInitState, cx: &mut App) -> Self {
        Self {
            fixture_selection: cx.new(|_| frontend_state.fixture_selection),
            fixture_values: cx.new(|_| {
                frontend_state
                    .fixture_states
                    .into_iter()
                    .map(|(id, state)| {
                        (
                            id,
                            state
                                .cached_output_moved()
                                .into_iter()
                                .map(|(channel, value)| (channel, value.value))
                                .collect(),
                        )
                    })
                    .collect()
            }),
            patch: cx.new(|_| frontend_state.patch),
        }
    }

    pub fn update_from_event(&self, event: DemexEvent, cx: &mut App) {
        match event {
            DemexEvent::FixtureSelectionChanged(new_selection) => {
                self.fixture_selection.update(cx, |sel, cx| {
                    *sel = new_selection;
                    cx.notify();
                })
            }
            _ => {}
        }
    }

    pub fn update_from_tick(&self, _tick: DemexEngineTickState, _cx: &mut App) {}

    pub fn update_fixture_values(
        &self,
        update: HashMap<u32, HashMap<String, FixtureChannelValue3>>,
        cx: &mut App,
    ) {
        self.fixture_values.update(cx, |fixtures, cx| {
            for (id, values) in update.into_iter() {
                fixtures.entry(id).and_modify(|fixture| {
                    for (channel, value) in values {
                        fixture.insert(channel, value);
                    }
                });
            }

            println!("fixture with id 1: {:?}", fixtures.get(&1));
            cx.notify();
        });
    }
}

impl Global for DemexUiState {}
