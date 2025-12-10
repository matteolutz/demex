use std::sync::{Arc, mpsc};

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
            FixtureNameRequest, PoolItemRequest, ShowRequest, ThreadStatsRequest,
        },
        component::ComponentHandle,
        state::{DemexEngineState, DemexFrontendInitState},
    },
    patch::Patch,
    show::DemexShow,
    thread::{
        DemexThread, DemexThreadHandle, debug::DebugThread, output::OutputThread,
        update::UpdateThread,
    },
    utils::thread::DemexThreadStatsHandler,
};

pub mod comm;
pub mod component;
pub mod error;
pub mod state;
pub mod tick;

pub struct DemexEngine {
    debug_thread: Option<DemexThreadHandle<DebugThread>>,
    stats: ComponentHandle<DemexThreadStatsHandler>,

    patch: Arc<ArcSwap<Patch>>,
    state: ComponentHandle<DemexEngineState>,
    action_queue: ComponentHandle<ActionQueue>,

    event_bus_tx: mpsc::Sender<DemexEngineCommEvent>,

    update_thread: Option<DemexThreadHandle<UpdateThread>>,
    output_thread: Option<DemexThreadHandle<OutputThread>>,
}

impl DemexEngine {
    pub fn new(event_bus_tx: mpsc::Sender<DemexEngineCommEvent>, start_debug: bool) -> Self {
        let stats = ComponentHandle::create_default();

        let s = Self {
            debug_thread: start_debug
                .then(|| DemexThread::start(DebugThread::default(), stats.clone())),
            stats,
            action_queue: ComponentHandle::create_default(),
            state: ComponentHandle::create_default(),
            event_bus_tx,
            patch: Arc::new(ArcSwap::from_pointee(Patch::default())),
            update_thread: None,
            output_thread: None,
        };

        s
    }
}

impl DemexEngine {
    pub fn load_show(
        &mut self,
        show: DemexShow,
        fixture_types: Vec<FixtureType>,
    ) -> (DemexEngineCommRequestDispatcher, DemexFrontendInitState) {
        self.stop_threads();

        let patch = show.patch.into_patch(fixture_types);
        self.patch.store(Arc::new(patch.clone()));

        let (tx, rx) = mpsc::channel();
        let mut comm_handler = DemexEngineCommRequestHandler::new(rx);
        let comm_dispatcher = DemexEngineCommRequestDispatcher::new(tx);

        self.register_comm_handlers(&mut comm_handler);

        let (value_queue_tx, value_queue_rx) = mpsc::channel();

        let (update_thread_delegate, fixture_states, pools) = UpdateThread::new(
            self.event_bus_tx.clone(),
            comm_handler,
            self.action_queue.clone(),
            value_queue_tx,
            show.preset_handler,
            show.updatable_handler,
            show.timing_handler,
            self.patch.clone(),
        );
        let update_thread = DemexThread::start(update_thread_delegate, self.stats());
        self.update_thread = Some(update_thread);

        let output_thread = DemexThread::start(
            OutputThread::new(self.patch.clone(), value_queue_rx),
            self.stats(),
        );
        self.output_thread = Some(output_thread);

        let frontend_state = DemexFrontendInitState {
            fixture_selection: self.state.read(|s| s.fixture_selection.clone()),
            fixture_states,
            pools,
            patch,
        };

        (comm_dispatcher, frontend_state)
    }

    fn stop_threads(&mut self) {
        if let Some(update_thread) = self.update_thread.take() {
            update_thread
                .stop_and_join()
                .expect("Should stop update thread");
        }

        if let Some(output_thread) = self.output_thread.take() {
            output_thread
                .stop_and_join()
                .expect("Should stop output thread");
        }
    }

    pub fn stop(mut self) {
        self.stop_threads();
        if let Some(debug_thread) = self.debug_thread.take() {
            debug_thread
                .stop_and_join()
                .expect("Should stop debug thread");
        }
    }

    fn register_comm_handlers(&self, handler: &mut DemexEngineCommRequestHandler) {
        handler.register(|FixtureNameRequest(id): FixtureNameRequest, payload| {
            payload.patch.fixture(id).map(|f| f.name.clone()).ok()
        });
        handler.register(|_: ThreadStatsRequest, payload| {
            payload.stats.read(|stats| stats.stats().clone())
        });

        handler.register(|_: ShowRequest, payload| payload.show.clone_into_show());
        handler.register(
            |PoolItemRequest { pool_type, id }: PoolItemRequest, payload| {
                let pool = payload.show.get_pool(pool_type);
                pool.get(pool_type, id).ok()
            },
        );
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
