use std::{
    sync::{Arc, mpsc},
    thread::JoinHandle,
};

use arc_swap::ArcSwap;
use gdtf::fixture_type::FixtureType;

use crate::{
    command::{
        lexer::Lexer,
        parser::{
            Parser2,
            nodes::action::{Action, ActionIssuer, queue::ActionQueue},
        },
    },
    engine::{
        comm::{
            DemexEngineCommEvent, DemexEngineCommRequestDispatcher, DemexEngineCommRequestHandler,
            FixtureNameRequest, ThreadStatsRequest,
        },
        component::ComponentHandle,
        state::DemexEngineState,
    },
    patch::Patch,
    show::DemexShow,
    utils::thread::DemexThreadStatsHandler,
};

pub mod comm;
pub mod component;
pub mod error;
pub mod state;
mod threads;
pub mod tick;

pub struct DemexEngine {
    stats: ComponentHandle<DemexThreadStatsHandler>,

    patch: ArcSwap<Patch>,
    state: ComponentHandle<DemexEngineState>,
    action_queue: ComponentHandle<ActionQueue>,

    event_bus_tx: mpsc::Sender<DemexEngineCommEvent>,
    threads: Vec<JoinHandle<()>>,
}

impl DemexEngine {
    pub fn new(event_bus_tx: mpsc::Sender<DemexEngineCommEvent>) -> Self {
        let s = Self {
            stats: ComponentHandle::create_default(),
            action_queue: ComponentHandle::create_default(),
            state: ComponentHandle::create_default(),
            event_bus_tx,
            threads: Vec::new(),
            patch: ArcSwap::from_pointee(Patch::default()),
        };

        s
    }
}

impl DemexEngine {
    pub fn load_show(
        &mut self,
        show: DemexShow,
        fixture_types: Vec<FixtureType>,
        start_debug: bool,
    ) -> DemexEngineCommRequestDispatcher {
        let patch = show.patch.into_patch(fixture_types);
        self.patch.store(Arc::new(patch));

        let (tx, rx) = mpsc::channel();
        let mut comm_handler = DemexEngineCommRequestHandler::new(rx);
        let comm_dispatcher = DemexEngineCommRequestDispatcher::new(tx);

        Self::register_comm_handlers(&mut comm_handler);

        let (value_queue_tx, value_queue_rx) = mpsc::channel();

        let update_thread = threads::update::start_demex_update_thread(
            self.event_bus_tx.clone(),
            comm_handler,
            self.stats(),
            self.action_queue.clone(),
            value_queue_tx,
            show.preset_handler,
            show.updatable_handler,
            show.timing_handler,
            &self.patch,
        );
        self.register_thread(update_thread);

        let output_thread = threads::output::start_demex_output_thread(
            self.stats.clone(),
            &self.patch,
            value_queue_rx,
        );
        self.register_thread(output_thread);

        if start_debug {
            let debug_thread = threads::debug::start_demex_debug_thread(self.stats());
            self.register_thread(debug_thread);
        }

        comm_dispatcher
    }

    fn register_thread(&mut self, join_handle: JoinHandle<()>) {
        self.threads.push(join_handle);
    }

    fn register_comm_handlers(handler: &mut DemexEngineCommRequestHandler) {
        handler.register(|FixtureNameRequest(id): FixtureNameRequest, payload| {
            payload.patch.fixture(id).map(|f| f.name.clone()).ok()
        });
        handler.register(|_: ThreadStatsRequest, payload| {
            payload.stats.read(|stats| stats.stats().clone())
        });
    }

    pub fn join_threads(&mut self) {
        for thread in self.threads.drain(..) {
            thread.join().unwrap();
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
    pub fn state(&self) -> ComponentHandle<DemexEngineState> {
        self.state.clone()
    }

    #[inline]
    pub fn stats(&self) -> ComponentHandle<DemexThreadStatsHandler> {
        self.stats.clone()
    }

    #[inline]
    pub fn read_patch(&self) -> Arc<Patch> {
        self.patch.load_full()
    }
}
