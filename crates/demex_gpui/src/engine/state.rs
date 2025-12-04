use std::{collections::HashMap, time::Duration};

use demex_core::{
    channel3::channel_value::FixtureChannelValue3,
    engine::{comm::ThreadStatsRequest, state::DemexFrontendInitState, tick::DemexEngineTickState},
    event::DemexEvent,
    patch::Patch,
    selection::FixtureSelection,
    utils::thread::DemexThreadStats,
};
use gpui::{App, AppContext, Entity, Global, Timer};

use crate::engine::DemexEngineHandler;

#[derive(Clone)]
pub struct DemexPerformanceBuffer<const SIZE: usize> {
    buffer: [Option<DemexThreadStats>; SIZE],
    write_cursor: usize,
    clear_on_wrap: bool,
}

impl<const SIZE: usize> Default for DemexPerformanceBuffer<SIZE> {
    fn default() -> Self {
        Self::new(false)
    }
}

impl<const SIZE: usize> DemexPerformanceBuffer<SIZE> {
    pub fn new(clear_on_wrap: bool) -> Self {
        Self {
            buffer: [const { None }; SIZE],
            write_cursor: 0,
            clear_on_wrap,
        }
    }

    pub fn add(&mut self, entry: DemexThreadStats) {
        if self.write_cursor >= SIZE {
            self.write_cursor = 0;

            if self.clear_on_wrap {
                self.buffer.iter_mut().for_each(|e| *e = None);
            }
        }

        self.buffer[self.write_cursor] = Some(entry);
        self.write_cursor += 1;
    }

    pub fn current_its(&self) -> Option<f64> {
        if self.write_cursor == 0 {
            None
        } else {
            self.buffer[self.write_cursor - 1]
                .as_ref()
                .map(|p| 1.0 / p.dt())
        }
    }

    pub fn data(&self) -> impl Iterator<Item = Option<&DemexThreadStats>> {
        self.buffer.iter().map(|s| s.as_ref())
    }

    pub fn move_data(self) -> impl Iterator<Item = Option<DemexThreadStats>> {
        self.buffer.into_iter()
    }

    pub fn write_cursor(&self) -> usize {
        self.write_cursor
    }
}

pub struct DemexUiState {
    fixture_selection: Entity<Option<FixtureSelection>>,
    fixture_values: Entity<HashMap<u32, HashMap<String, FixtureChannelValue3>>>,
    patch: Entity<Patch>,

    performance: Entity<HashMap<String, DemexPerformanceBuffer<10>>>,

    command_history: Entity<Vec<String>>,
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

    pub fn patch(cx: &App) -> Entity<Patch> {
        let this: &Self = cx.global();
        this.patch.clone()
    }

    pub fn performance(cx: &App) -> Entity<HashMap<String, DemexPerformanceBuffer<10>>> {
        let this: &Self = cx.global();
        this.performance.clone()
    }

    pub fn command_history(cx: &App) -> Entity<Vec<String>> {
        let this: &Self = cx.global();
        this.command_history.clone()
    }
}

impl DemexUiState {
    pub(super) fn start_performance_thread(cx: &mut App) {
        cx.spawn(async move |cx| {
            loop {
                let _ = cx.update_global(|ui_state: &mut Self, cx| {
                    DemexEngineHandler::send_with(
                        cx,
                        ui_state.performance.clone(),
                        ThreadStatsRequest {},
                        |res, this, cx| {
                            res.into_iter().for_each(|(thread, stat)| {
                                this.entry(thread).or_default().add(stat)
                            });

                            cx.notify();
                        },
                    )
                });

                Timer::after(Duration::from_millis(500)).await;
            }
        })
        .detach();
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
            performance: cx.new(|_| HashMap::new()),
            command_history: cx.new(|_| Vec::new()),
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
            _ => Default::default(),
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

    pub fn update_patch(&self, new_patch: Patch, cx: &mut App) {
        self.patch.update(cx, |patch, cx| {
            *patch = new_patch;
            cx.notify();
        })
    }
}

impl Global for DemexUiState {}
