use std::{collections::HashMap, sync::Arc};

use demex_core::{
    engine::tick::DemexEngineTickState, event::DemexEvent, patch::Patch,
    selection::FixtureSelection,
};
use gpui::{App, AppContext, Entity, Global};

pub struct DemexUiState {
    fixture_selection: Entity<Option<FixtureSelection>>,
    fixture_values: Entity<HashMap<u32, HashMap<String, f32>>>,
    patch: Entity<Arc<Patch>>,
}

impl DemexUiState {
    pub fn fixture_selection(cx: &App) -> Entity<Option<FixtureSelection>> {
        let this: &Self = cx.global();
        this.fixture_selection.clone()
    }

    pub fn fixture_values(cx: &App) -> Entity<HashMap<u32, HashMap<String, f32>>> {
        let this: &Self = cx.global();
        this.fixture_values.clone()
    }

    pub fn patch(cx: &App) -> Entity<Arc<Patch>> {
        let this: &Self = cx.global();
        this.patch.clone()
    }
}

impl DemexUiState {
    pub fn new(cx: &mut App) -> Self {
        Self {
            fixture_selection: cx.new(|_| None),
            fixture_values: cx.new(|_| HashMap::new()),
            patch: cx.new(|_| Arc::new(Patch::default())),
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
}

impl Global for DemexUiState {}
