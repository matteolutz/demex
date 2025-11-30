use std::sync::mpsc;

use demex_core::{
    engine::{
        DemexEngine,
        comm::{DemexEngineCommRequest, DemexEngineCommRequestDispatcher},
        error::DemexEngineError,
    },
    show::DemexShow,
};
use gdtf::fixture_type::FixtureType;
use gpui::{App, AppContext, AsyncApp, Context, Entity, Global};

use crate::engine::{event::DemexEventHandler, state::DemexUiState};

pub mod event;
pub mod state;

pub struct DemexEngineHandler {
    engine: DemexEngine,

    event_handler: Entity<DemexEventHandler>,

    dispatcher: DemexEngineCommRequestDispatcher,
}

impl DemexEngineHandler {
    pub fn init(
        global_fixture_types: Vec<FixtureType>,
        show: DemexShow,
        cx: &mut App,
    ) -> Result<(), DemexEngineError> {
        let (event_bus_tx, event_bus_rx) = mpsc::channel();

        let mut engine = DemexEngine::new(event_bus_tx);

        let ui_state = DemexUiState::new(cx);
        cx.set_global(ui_state);

        let event_handler = cx.new(|cx| DemexEventHandler::new(event_bus_rx, cx));

        let dispatcher = engine.load_show(show, global_fixture_types, false);

        let patch = engine.read_patch();
        DemexUiState::patch(cx).update(cx, |ui_patch, _| *ui_patch = patch);

        cx.set_global(Self {
            engine,
            event_handler,
            dispatcher,
        });

        Ok(())
    }

    pub fn engine(cx: &App) -> &DemexEngine {
        let this: &Self = cx.global();
        &this.engine
    }

    pub fn event_handler(cx: &App) -> Entity<DemexEventHandler> {
        let this: &Self = cx.global();
        this.event_handler.clone()
    }

    pub fn send<R, CB>(cx: &mut App, req: R, cb: CB)
    where
        R: DemexEngineCommRequest,
        CB: FnOnce(R::Response, &mut App) + Send + 'static,
    {
        let this: &Self = cx.global();
        let rx_resp = this.dispatcher.send(req);

        cx.spawn(async move |cx: &mut AsyncApp| {
            let boxed = rx_resp.recv().unwrap();
            let res = *boxed.downcast::<R::Response>().unwrap();

            let _ = cx.update(|cx| {
                cb(res, cx);
            });
        })
        .detach();
    }

    pub fn send_with<R, T, CB>(cx: &mut App, entity: Entity<T>, req: R, cb: CB)
    where
        R: DemexEngineCommRequest,
        T: 'static,
        CB: FnOnce(R::Response, &mut T, &mut Context<T>) + Send + 'static,
    {
        let this: &Self = cx.global();
        let rx_resp = this.dispatcher.send(req);

        cx.spawn(async move |cx: &mut AsyncApp| {
            let boxed = rx_resp.recv().unwrap();
            let res = *boxed.downcast::<R::Response>().unwrap();

            let _ = entity.update(cx, |entity, cx| {
                cb(res, entity, cx);
            });
        })
        .detach();
    }
}

impl Global for DemexEngineHandler {}
