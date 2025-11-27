/*
 *
 * This file has been modified from its original version.
 * Original: https://github.com/BaukeWestendorp/radiant
 * License: Apache 2.0 - https://github.com/BaukeWestendorp/radiant/blob/main/LICENCE
 *
 */

use std::{
    any::{Any, TypeId},
    collections::{HashMap, VecDeque},
    sync::{Arc, mpsc},
};

use parking_lot::Mutex;

use crate::{
    channel3::channel_value_queue::{ChannelValueQueue, ChannelValueQueueEntry},
    command::{
        lexer::Lexer,
        parser::{
            Parser2,
            nodes::action::{Action, ActionIssuer, queue::ActionQueue},
        },
    },
    engine::{
        component::{Component, ComponentHandle},
        state::DemexEngineState,
    },
    event::DemexEvent,
    fixture::handler::FixtureHandler,
    input::event::handler::DemexInputDeviceEventHandler,
    patch::Patch,
    presets::PresetHandler,
    timing::TimingHandler,
    updatables::UpdatableHandler,
    utils::thread::DemexThreadStatsHandler,
};

pub mod component;
pub mod error;
pub mod state;
pub mod threads;

pub struct DemexEngine {
    components: HashMap<TypeId, Arc<Mutex<dyn Any + Send + Sync>>>,
    stats: ComponentHandle<DemexThreadStatsHandler>,

    action_queue: ComponentHandle<ActionQueue>,
    value_queue: ComponentHandle<ChannelValueQueue>,

    state: ComponentHandle<DemexEngineState>,

    event_bus_tx: mpsc::Sender<DemexEvent>,
}

impl DemexEngine {
    pub fn new(event_bus_tx: mpsc::Sender<DemexEvent>) -> Self {
        Self {
            components: HashMap::new(),
            stats: ComponentHandle::create_default(),
            action_queue: ComponentHandle::create_default(),
            state: ComponentHandle::create_default(),
            value_queue: ComponentHandle::create_default(),
            event_bus_tx,
        }
    }
}

impl DemexEngine {
    pub fn register_component<T>(&mut self, component: T)
    where
        T: Component + Send + Sync + 'static,
    {
        let type_id = TypeId::of::<T>();
        self.components
            .insert(type_id, Arc::new(Mutex::new(component)));
    }

    pub fn component<T: Component + 'static>(&self) -> ComponentHandle<T> {
        let type_id = TypeId::of::<T>();
        let component = self
            .components
            .get(&type_id)
            .expect(format!("Component {:?} not registered", type_id).as_str());
        ComponentHandle::new(component.clone())
    }

    pub fn start(&mut self, start_debug: bool) {
        threads::update::start_demex_update_thread(
            self.event_bus_tx.clone(),
            self.stats(),
            self.action_queue(),
            self.fixture_handler(),
            self.preset_handler(),
            self.updatable_handler(),
            self.timing_handler(),
            self.patch(),
            self.input_device_event_handler(),
            self.state(),
        );

        threads::output::start_demex_output_thread(
            self.stats(),
            self.fixture_handler(),
            self.preset_handler(),
            self.timing_handler(),
            self.patch(),
        );

        if start_debug {
            threads::debug::start_demex_debug_thread(self.stats());
        }
    }

    pub fn exec_command(&self, command: &str) -> Result<(), Box<dyn std::error::Error>> {
        let now = std::time::Instant::now();

        let mut lexer = Lexer::new(command);
        let tokens = lexer.tokenize()?;

        let mut parser = Parser2::new(&tokens);
        let action = parser.parse()?;

        self.action_queue()
            .write(|action_queue| action_queue.enqueue_at(action, now, ActionIssuer::Command));

        Ok(())
    }

    pub fn exec_ui(&self, action: Action) {
        self.action_queue()
            .write(|action_queue| action_queue.enqueue_now(action, ActionIssuer::Ui));
    }

    #[inline]
    pub fn action_queue(&self) -> ComponentHandle<ActionQueue> {
        self.action_queue.clone()
    }

    #[inline]
    pub fn fixture_handler(&self) -> ComponentHandle<FixtureHandler> {
        self.component()
    }

    #[inline]
    pub fn preset_handler(&self) -> ComponentHandle<PresetHandler> {
        self.component()
    }

    #[inline]
    pub fn updatable_handler(&self) -> ComponentHandle<UpdatableHandler> {
        self.component()
    }

    #[inline]
    pub fn timing_handler(&self) -> ComponentHandle<TimingHandler> {
        self.component()
    }

    #[inline]
    pub fn patch(&self) -> ComponentHandle<Patch> {
        self.component()
    }

    #[inline]
    pub fn input_device_event_handler(&self) -> ComponentHandle<DemexInputDeviceEventHandler> {
        self.component()
    }

    #[inline]
    pub fn state(&self) -> ComponentHandle<DemexEngineState> {
        self.state.clone()
    }

    #[inline]
    pub fn stats(&self) -> ComponentHandle<DemexThreadStatsHandler> {
        self.stats.clone()
    }
}
