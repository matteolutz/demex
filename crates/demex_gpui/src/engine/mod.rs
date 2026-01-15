use std::sync::mpsc;

use demex_core::{
    engine::{
        DemexEngine,
        comm::{DemexEngineCommRequest, DemexEngineCommRequestDispatcher},
        error::DemexEngineError,
        state::DemexFrontendInitState,
    },
    show::DemexShow,
};
use gdtf::fixture_type::FixtureType;
use gpui::{App, AppContext, AsyncApp, AsyncWindowContext, Context, Entity, Global, Window};

use crate::engine::event::DemexEventHandler;

pub mod event;
pub mod showfile;
pub mod state;

pub struct DemexEngineHandler {
    engine: DemexEngine,

    event_handler: Entity<DemexEventHandler>,

    dispatcher: DemexEngineCommRequestDispatcher,
}

impl DemexEngineHandler {
    pub fn init(
        show: DemexShow,
        fixture_types: Vec<FixtureType>,
        cx: &mut App,
    ) -> Result<DemexFrontendInitState, DemexEngineError> {
        let (event_bus_tx, event_bus_rx) = mpsc::channel();

        let mut engine = DemexEngine::new(event_bus_tx, false);

        let (dispatcher, frontend_state) = engine.load_show(show, fixture_types);

        let event_handler = cx.new(|cx| DemexEventHandler::new(event_bus_rx, cx));

        cx.set_global(Self {
            engine,
            event_handler,
            dispatcher,
        });

        Ok(frontend_state)
    }

    fn update_show(
        &mut self,
        show: DemexShow,
        fixture_types: Vec<FixtureType>,
    ) -> DemexFrontendInitState {
        let (dispatcher, frontend_state) = self.engine.load_show(show, fixture_types);
        self.dispatcher = dispatcher;
        frontend_state
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

    pub fn send_in<R, CB>(window: &mut Window, cx: &mut App, req: R, cb: CB)
    where
        R: DemexEngineCommRequest,
        CB: FnOnce(R::Response, &mut Window, &mut App) + Send + 'static,
    {
        let this: &Self = cx.global();
        let rx_resp = this.dispatcher.send(req);

        window
            .spawn(cx, async move |cx: &mut AsyncWindowContext| {
                let boxed = rx_resp.recv().unwrap();
                let res = *boxed.downcast::<R::Response>().unwrap();

                let _ = cx.update(|window, cx| {
                    cb(res, window, cx);
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
