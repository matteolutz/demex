use std::sync::{Arc, mpsc};

use arc_swap::ArcSwap;
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

    command_input: Arc<ArcSwap<String>>,

    dispatcher: DemexEngineCommRequestDispatcher,
}

impl DemexEngineHandler {
    pub fn init(
        show: DemexShow,
        fixture_types: Vec<FixtureType>,
        cx: &mut App,
    ) -> Result<DemexFrontendInitState, DemexEngineError> {
        let (event_bus_tx, event_bus_rx) = mpsc::channel();

        let command_input = Arc::new(ArcSwap::from_pointee(String::new()));

        let mut engine = DemexEngine::new(event_bus_tx, command_input.clone(), false);

        let (dispatcher, frontend_state) = engine.load_show(show, fixture_types);

        let event_handler = cx.new(|cx| DemexEventHandler::new(event_bus_rx, cx));

        cx.set_global(Self {
            engine,
            event_handler,
            command_input,
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

    pub fn update_command_input(value: impl Into<String>, cx: &mut App) {
        let value = value.into();
        let this: &Self = cx.global();
        this.command_input.store(Arc::new(value));
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

    pub fn send_in_visual<T: 'static, R, CB>(
        window: &mut Window,
        cx: &mut Context<T>,
        req: R,
        cb: CB,
    ) where
        R: DemexEngineCommRequest,
        CB: FnOnce(&mut T, R::Response, &mut Window, &mut Context<T>) + Send + 'static,
    {
        let this: &Self = cx.global();
        let rx_resp = this.dispatcher.send(req);

        let entity = cx.entity();

        window
            .spawn(cx, async move |cx: &mut AsyncWindowContext| {
                let boxed = rx_resp.recv().unwrap();
                let res = *boxed.downcast::<R::Response>().unwrap();

                let _ = cx.update(|window, cx| {
                    entity.update(cx, |entity, cx| {
                        cb(entity, res, window, cx);
                    });
                });
            })
            .detach();
    }

    pub fn send_with<R, T, CB>(entity: Entity<T>, cx: &mut App, req: R, cb: CB)
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
