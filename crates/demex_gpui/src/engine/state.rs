use std::{
    collections::{HashMap, VecDeque},
    rc::Rc,
    time::{self, Duration},
};

use demex_core::{
    channel3::{attribute::FixtureChannel3Attribute, channel_value::FixtureChannelValue3},
    command::parser::nodes::object::ObjectDelegate,
    engine::{
        comm::{PoolItemRequest, ThreadStatsRequest},
        state::DemexFrontendInitState,
        tick::DemexEngineTickState,
    },
    event::{DemexEvent, FixtureSelectionWithGroup},
    fixture::FixturePath,
    patch::Patch,
    pool::{PoolItem, PoolType},
    timing::speed_master::SpeedMasterValue,
    utils::thread::DemexThreadStats,
};
use gpui::{
    AnyWindowHandle, App, AppContext, BorrowAppContext, Context, Entity, Global, Subscription,
    Window,
};
use gpui_component::input::{InputEvent, InputState, Position};

use crate::engine::DemexEngineHandler;

#[derive(Debug, Clone)]
pub struct DemexCommandHistoryEntry {
    pub command: String,
    pub timestamp: chrono::DateTime<chrono::Local>,
    pub success: bool,
}

impl DemexCommandHistoryEntry {
    pub fn now(command: String, success: bool) -> Self {
        Self {
            command,
            timestamp: chrono::Local::now(),
            success,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct DemexCommandHistory {
    history: VecDeque<DemexCommandHistoryEntry>,
    max_len: usize,
}

impl DemexCommandHistory {
    pub fn new(max_len: usize) -> Self {
        Self {
            history: VecDeque::with_capacity(max_len),
            max_len,
        }
    }

    pub fn len(&self) -> usize {
        self.history.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &DemexCommandHistoryEntry> {
        self.history.iter()
    }

    pub fn push_now(&mut self, command: String, success: bool) {
        self.history
            .retain(|prev_command| prev_command.command != command);

        if self.history.len() == self.max_len {
            self.history.pop_front();
        }

        self.history
            .push_back(DemexCommandHistoryEntry::now(command, success));
    }

    pub fn get(&self, offset: usize) -> Option<&DemexCommandHistoryEntry> {
        let idx = self.history.len() - offset - 1;
        self.history.get(idx)
    }

    pub fn clear(&mut self) {
        self.history.clear();
    }
}

#[derive(Debug, Clone)]
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

#[derive(Clone)]
pub struct DemexUiCommandInputState {
    input_state: Entity<InputState>,
    window_handle: AnyWindowHandle,

    _subscription: Rc<Subscription>,
}

impl DemexUiCommandInputState {
    pub fn new(
        input_state: Entity<InputState>,
        window_handle: AnyWindowHandle,
        cx: &mut App,
    ) -> Self {
        let subscription =
            cx.subscribe(&input_state, move |this, evt: &InputEvent, cx| match evt {
                InputEvent::Change => {
                    DemexEngineHandler::update_command_input(this.read(cx).value(), cx)
                }
                _ => {}
            });

        Self {
            input_state,
            window_handle,
            _subscription: Rc::new(subscription),
        }
    }

    fn set_cursor_end(state: &mut InputState, window: &mut Window, cx: &mut Context<InputState>) {
        state.set_cursor_position(Position::new(0, state.value().len() as u32), window, cx);
    }

    pub fn focus(self, cx: &mut App) {
        cx.defer(move |cx| {
            let _ = self.window_handle.update(cx, |_, window, cx| {
                window.activate_window();

                self.input_state.update(cx, |state, cx| {
                    Self::set_cursor_end(state, window, cx);
                    state.focus(window, cx);
                });
            });
        });
    }

    pub fn append(&self, text: impl ToString, cx: &mut App) {
        let _ = self.window_handle.update(cx, |_, window, cx| {
            window.activate_window();

            self.input_state.update(cx, |state, cx| {
                let value = state.value();
                let value = value.strip_suffix(" ").unwrap_or(value.as_str());

                let new_value = if value.is_empty() {
                    text.to_string()
                } else {
                    format!("{} {} ", value, text.to_string())
                };

                state.set_value(new_value, window, cx);
                Self::set_cursor_end(state, window, cx);
                state.focus(window, cx);
            });
        });
    }
}

#[derive(Debug, Default, Clone)]
pub struct DemexUiSpeedMasterValue {
    pub bpm: f32,
    pub last_tap: Option<time::Instant>,
}

impl From<SpeedMasterValue> for DemexUiSpeedMasterValue {
    fn from(value: SpeedMasterValue) -> Self {
        Self {
            bpm: value.bpm(),
            last_tap: value.interval(),
        }
    }
}

pub struct DemexUiState {
    fixture_selection: Entity<Option<FixtureSelectionWithGroup>>,
    highlight: Entity<Option<FixtureSelectionWithGroup>>,

    fixture_values:
        Entity<HashMap<FixturePath, HashMap<FixtureChannel3Attribute, FixtureChannelValue3>>>,
    patch: Entity<Patch>,

    performance: Entity<HashMap<String, DemexPerformanceBuffer<10>>>,

    pools: HashMap<PoolType, Entity<Vec<PoolItem>>>,

    speed_masters: Entity<HashMap<u32, DemexUiSpeedMasterValue>>,

    command_input_state: Option<DemexUiCommandInputState>,
    command_history: Entity<DemexCommandHistory>,

    selected_sequence: Entity<Option<u32>>,
}

impl DemexUiState {
    pub fn read(cx: &App) -> &Self {
        cx.global()
    }

    pub fn update(cx: &mut App, cb: impl FnOnce(&mut Self, &mut App)) {
        cx.update_global(cb)
    }

    pub fn fixture_selection(cx: &App) -> Entity<Option<FixtureSelectionWithGroup>> {
        let this: &Self = cx.global();
        this.fixture_selection.clone()
    }

    pub fn highlight(cx: &App) -> Entity<Option<FixtureSelectionWithGroup>> {
        let this: &Self = cx.global();
        this.highlight.clone()
    }

    pub fn fixture_values(
        cx: &App,
    ) -> Entity<HashMap<FixturePath, HashMap<FixtureChannel3Attribute, FixtureChannelValue3>>> {
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

    pub fn command_history(cx: &App) -> Entity<DemexCommandHistory> {
        let this: &Self = cx.global();
        this.command_history.clone()
    }

    fn get_or_insert_pool(
        &mut self,
        pool_type: PoolType,
        cx: &mut App,
    ) -> &mut Entity<Vec<PoolItem>> {
        self.pools
            .entry(pool_type)
            .or_insert_with(|| cx.new(|_| Vec::new()))
    }

    pub fn pool(pool_type: PoolType, cx: &mut App) -> Entity<Vec<PoolItem>> {
        cx.update_global(|this: &mut Self, cx| this.get_or_insert_pool(pool_type, cx).clone())
    }

    pub fn try_pool(pool_type: PoolType, cx: &App) -> Option<Entity<Vec<PoolItem>>> {
        let this: &Self = cx.global();
        this.pools.get(&pool_type).cloned()
    }

    pub fn speed_masters(cx: &App) -> Entity<HashMap<u32, DemexUiSpeedMasterValue>> {
        let this: &Self = cx.global();
        this.speed_masters.clone()
    }

    pub fn selected_sequence(cx: &App) -> Entity<Option<u32>> {
        let this: &Self = cx.global();
        this.selected_sequence.clone()
    }

    pub fn set_command_input_state(
        input_state: &Entity<InputState>,
        window: &mut Window,
        cx: &mut App,
    ) {
        let state = DemexUiCommandInputState::new(input_state.clone(), window.window_handle(), cx);

        let this: &mut Self = cx.global_mut();
        this.command_input_state = Some(state);
    }

    pub fn command_input_state(cx: &App) -> Option<DemexUiCommandInputState> {
        let this: &Self = cx.global();
        this.command_input_state.as_ref().cloned()
    }

    pub fn update_command_input_state(
        cx: &mut App,
        f: impl FnOnce(DemexUiCommandInputState, &mut App),
    ) {
        if let Some(command_input_state) = DemexUiState::command_input_state(cx) {
            f(command_input_state, cx);
        }
    }
}

impl DemexUiState {
    pub fn init(cx: &mut App) {
        let ui_state = DemexUiState::new(cx);
        cx.set_global(ui_state);
    }

    pub fn start_performance_thread(cx: &mut App) {
        cx.spawn(async move |cx| {
            loop {
                let _ = cx.update_global(|ui_state: &mut Self, cx| {
                    DemexEngineHandler::send_with(
                        ui_state.performance.clone(),
                        cx,
                        ThreadStatsRequest {},
                        |res, this, cx| {
                            res.into_iter().for_each(|(thread, stat)| {
                                this.entry(thread).or_default().add(stat)
                            });

                            cx.notify();
                        },
                    )
                });

                cx.background_executor()
                    .timer(Duration::from_millis(500))
                    .await;
            }
        })
        .detach();
    }
}

impl DemexUiState {
    pub fn new(cx: &mut App) -> Self {
        Self {
            fixture_selection: cx.new(|_| Default::default()),
            highlight: cx.new(|_| Default::default()),
            fixture_values: cx.new(|_| Default::default()),
            patch: cx.new(|_| Default::default()),
            performance: cx.new(|_| Default::default()),
            pools: HashMap::new(),
            command_history: cx.new(|_| Default::default()),
            selected_sequence: cx.new(|_| None),
            speed_masters: cx.new(|_| Default::default()),
            command_input_state: None,
        }
    }

    pub fn load_frontend_state(&mut self, frontend_state: DemexFrontendInitState, cx: &mut App) {
        self.fixture_selection.update(cx, |sel, cx| {
            *sel = frontend_state.fixture_selection.map(|sel| sel.into());
            cx.notify();
        });

        self.fixture_values.update(cx, |values, cx| {
            *values = frontend_state
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
                .collect();
            cx.notify();
        });

        self.patch.update(cx, |patch, cx| {
            *patch = frontend_state.patch;
            cx.notify();
        });

        // don't update performance

        self.command_history.update(cx, |history, cx| {
            history.clear();
            cx.notify();
        });

        for (pool_type, pool_items) in frontend_state.pools.into_iter() {
            let pool = self
                .pools
                .entry(pool_type)
                .or_insert_with(|| cx.new(|_| Vec::new()));
            pool.update(cx, |pool, cx| {
                *pool = pool_items;
                cx.notify();
            })
        }

        self.speed_masters.update(cx, |speed_masters, cx| {
            speed_masters.clear();
            speed_masters.extend(
                frontend_state
                    .speedmasters
                    .into_iter()
                    .map(|(id, value)| (id, value.into())),
            );
            cx.notify();
        });
    }

    fn update_pool_item(pool_type: PoolType, id: u32, cx: &mut App) {
        DemexEngineHandler::send(cx, PoolItemRequest { pool_type, id }, move |res, cx| {
            let Some(item) = res else {
                return;
            };

            cx.update_global(|this: &mut Self, cx| {
                let pool = this.get_or_insert_pool(pool_type, cx);
                pool.update(cx, |pool_items, cx| {
                    pool_items.retain(|item| item.id != id);
                    pool_items.push(item);
                    cx.notify();
                });
            })
        });
    }

    pub fn update_from_event(&mut self, event: DemexEvent, cx: &mut App) {
        match event {
            DemexEvent::FixtureSelectionChanged(new_selection) => {
                self.fixture_selection.update(cx, |sel, cx| {
                    *sel = new_selection;
                    cx.notify();
                })
            }
            DemexEvent::HighlightChanged(new_highlight) => self.highlight.update(cx, |hl, cx| {
                *hl = new_highlight;
                cx.notify();
            }),
            DemexEvent::ObjectPropertyChanged(object, _) => {
                if let Some((pool_type, id)) = object.get_pool_type_and_id() {
                    Self::update_pool_item(pool_type, id, cx);
                }
            }
            DemexEvent::PoolItemFlagsUpdated(pool_type, id) => {
                Self::update_pool_item(pool_type, id, cx)
            }
            DemexEvent::PoolItemAdded(pool_type, id) => {
                DemexEngineHandler::send(cx, PoolItemRequest { pool_type, id }, move |res, cx| {
                    let Some(item) = res else {
                        return;
                    };

                    cx.update_global(|this: &mut Self, cx| {
                        let pool = this.get_or_insert_pool(pool_type, cx);
                        pool.update(cx, |pool_items, cx| {
                            pool_items.push(item);
                            cx.notify();
                        });
                    })
                });
            }
            DemexEvent::PoolItemsDeleted {
                pool_type,
                from_id,
                to_id,
            } => {
                let pool = self.get_or_insert_pool(pool_type, cx);
                pool.update(cx, |pool, cx| {
                    pool.retain(|pool_item| pool_item.id < from_id || pool_item.id > to_id);
                    cx.notify();
                });
            }
            DemexEvent::PoolItemMoved {
                pool_type,
                from_id,
                to_id,
            } => {
                let pool = self.get_or_insert_pool(pool_type, cx);
                pool.update(cx, |pool, cx| {
                    let item = pool.iter_mut().find(|item| item.id == from_id);
                    let Some(item) = item else {
                        return;
                    };
                    item.id = to_id;
                    cx.notify();
                });
            }
            DemexEvent::SpeedmasterFaderValueChanged {
                speed_master_id,
                bpm,
            } => {
                self.speed_masters.update(cx, |speed_masters, cx| {
                    let value = speed_masters.entry(speed_master_id).or_default();
                    *value = DemexUiSpeedMasterValue {
                        bpm,
                        last_tap: None,
                    };
                    cx.notify();
                });
            }
            DemexEvent::SpeedmasterTapped {
                speed_master_id,
                instant,
            } => {
                self.speed_masters.update(cx, |speed_masters, cx| {
                    if let Some(value) = speed_masters.get_mut(&speed_master_id) {
                        value.last_tap = Some(instant);
                    }
                    cx.notify();
                });
            }
            _ => Default::default(),
        }
    }

    pub fn update_from_tick(&self, _tick: DemexEngineTickState, _cx: &mut App) {}

    pub fn update_fixture_values(
        &self,
        update: HashMap<FixturePath, HashMap<FixtureChannel3Attribute, FixtureChannelValue3>>,
        cx: &mut App,
    ) {
        self.fixture_values.update(cx, |fixtures, cx| {
            for (id, values) in update.into_iter() {
                let fixture_values = fixtures.entry(id).or_default();

                for (channel, value) in values {
                    fixture_values.insert(channel, value);
                }
            }

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
