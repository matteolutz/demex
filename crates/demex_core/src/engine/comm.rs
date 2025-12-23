use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::mpsc,
};

use crate::{
    channel3::{attribute::FixtureChannel3Attribute, channel_value::FixtureChannelValue3},
    command::parser::nodes::action::result::ActionRunResult,
    engine::{component::ComponentHandle, tick::DemexEngineTickState},
    event::DemexEvent,
    fixture::FixturePath,
    patch::Patch,
    pool::{PoolItem, PoolType},
    sequence::frontend::FrontendSequence,
    show::{DemexShow, DemexShowRef},
    utils::thread::{DemexThreadStats, DemexThreadStatsHandler},
};

#[derive(Debug, Clone)]
pub enum DemexEngineCommEvent {
    DemexEvent(DemexEvent),

    ActionRunResult(ActionRunResult),

    Error(String),

    TickStateUpdate(DemexEngineTickState),
    FixtureValuesUpdate(
        HashMap<FixturePath, HashMap<FixtureChannel3Attribute, FixtureChannelValue3>>,
    ),
}

impl From<DemexEvent> for DemexEngineCommEvent {
    fn from(event: DemexEvent) -> Self {
        DemexEngineCommEvent::DemexEvent(event)
    }
}

pub trait DemexEngineCommRequest: Send + 'static + std::fmt::Debug {
    type Response: Send + 'static;
}

// Test request
#[derive(Debug)]
pub struct FixtureNameRequest(pub FixturePath);
impl DemexEngineCommRequest for FixtureNameRequest {
    type Response = Option<String>;
}

#[derive(Debug)]
pub struct ThreadStatsRequest {}
impl DemexEngineCommRequest for ThreadStatsRequest {
    type Response = HashMap<String, DemexThreadStats>;
}

#[derive(Debug)]
pub struct ShowRequest {}
impl DemexEngineCommRequest for ShowRequest {
    type Response = DemexShow;
}

#[derive(Debug)]
pub struct PoolItemRequest {
    pub pool_type: PoolType,
    pub id: u32,
}
impl DemexEngineCommRequest for PoolItemRequest {
    type Response = Option<PoolItem>;
}

#[derive(Debug)]
pub struct SequenceRequest {
    pub sequence_id: u32,
}
pub struct SequenceResponse {
    pub sequence: FrontendSequence,
    pub first_executor: Option<u32>,
}
impl DemexEngineCommRequest for SequenceRequest {
    type Response = Option<SequenceResponse>;
}

pub struct DemexEngineCommRequestEnvelope {
    type_id: TypeId,
    payload: Box<dyn Any + Send>,
    responder: mpsc::Sender<Box<dyn Any + Send>>,
}

#[derive(Clone)]
pub struct DemexEngineCommRequestDispatcher {
    tx: mpsc::Sender<DemexEngineCommRequestEnvelope>,
}

impl DemexEngineCommRequestDispatcher {
    pub fn new(tx: mpsc::Sender<DemexEngineCommRequestEnvelope>) -> Self {
        Self { tx }
    }

    // -------------------------------------------------------
    // Send request from frontend; callback executes on UI thread
    // -------------------------------------------------------
    pub fn send<R>(&self, req: R) -> mpsc::Receiver<Box<dyn Any + Send + 'static>>
    where
        R: DemexEngineCommRequest,
    {
        let (tx_resp, rx_resp) = mpsc::channel::<Box<dyn Any + Send>>();

        let env = DemexEngineCommRequestEnvelope {
            type_id: TypeId::of::<R>(),
            payload: Box::new(req),
            responder: tx_resp,
        };

        self.tx.send(env).unwrap();

        rx_resp
    }
}

#[derive(Copy, Clone)]
pub(crate) struct DemexEngineCommRequestHandlerPayload<'a> {
    pub patch: &'a Patch,
    pub stats: &'a ComponentHandle<DemexThreadStatsHandler>,
    pub show: DemexShowRef<'a>,
}

pub(crate) struct DemexEngineCommRequestHandler {
    rx: mpsc::Receiver<DemexEngineCommRequestEnvelope>,
    handlers: HashMap<
        TypeId,
        Box<
            dyn Fn(Box<dyn Any + Send>, DemexEngineCommRequestHandlerPayload) -> Box<dyn Any + Send>
                + Send,
        >,
    >,
}

impl DemexEngineCommRequestHandler {
    pub fn new(rx: mpsc::Receiver<DemexEngineCommRequestEnvelope>) -> Self {
        Self {
            rx,
            handlers: HashMap::new(),
        }
    }

    pub fn register<R, F>(&mut self, handler: F)
    where
        R: DemexEngineCommRequest,
        F: Fn(R, DemexEngineCommRequestHandlerPayload) -> R::Response + Send + 'static,
    {
        let type_id = TypeId::of::<R>();

        let wrapper = move |boxed: Box<dyn Any + Send>,
                            payload: DemexEngineCommRequestHandlerPayload|
              -> Box<dyn Any + Send> {
            let req = *boxed.downcast::<R>().unwrap();
            let res = handler(req, payload);
            Box::new(res)
        };

        self.handlers.insert(type_id, Box::new(wrapper));
    }

    pub fn handle_all(&self, payload: DemexEngineCommRequestHandlerPayload) {
        for env in self.rx.try_iter() {
            let handler = self.handlers.get(&env.type_id).expect(
                format!("No handler registered for request type {:?}", env.type_id).as_str(),
            );

            let res = handler(env.payload, payload);
            let _ = env.responder.send(res);
        }
    }
}
