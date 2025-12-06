use std::{
    collections::HashMap,
    sync::{Arc, mpsc},
};

use arc_swap::ArcSwap;

use crate::{
    channel3::channel_value_queue::ChannelValueQueueEntry,
    command::parser::nodes::{
        action::{ActionIssuer, queue::ActionQueue, result::ActionRunResult},
        fixture_selector::FixtureSelectorContext,
    },
    engine::{
        comm::{
            DemexEngineCommEvent, DemexEngineCommRequestHandler,
            DemexEngineCommRequestHandlerPayload,
        },
        component::ComponentHandle,
        state::DemexEngineState,
    },
    event::DemexEvent,
    input::DemexInputDeviceHandler,
    patch::Patch,
    presets::PresetHandler,
    show::DemexShowRef,
    state::{fixture_state::FixtureState, fixture_state_handler::FixtureStateHandler},
    thread::DemexThreadDelegate,
    timing::TimingHandler,
    updatables::UpdatableHandler,
};

pub struct UpdateThread {
    event_bus_tx: mpsc::Sender<DemexEngineCommEvent>,
    request_handler: DemexEngineCommRequestHandler,
    action_queue: ComponentHandle<ActionQueue>,
    value_queue_tx: mpsc::Sender<ChannelValueQueueEntry>,
    preset_handler: PresetHandler,
    updatable_handler: UpdatableHandler,
    timing_handler: TimingHandler,
    patch: Arc<ArcSwap<Patch>>,

    fixture_state_handler: FixtureStateHandler,
    state: DemexEngineState,
}

impl UpdateThread {
    pub fn new(
        event_bus_tx: mpsc::Sender<DemexEngineCommEvent>,
        request_handler: DemexEngineCommRequestHandler,
        action_queue: ComponentHandle<ActionQueue>,
        value_queue_tx: mpsc::Sender<ChannelValueQueueEntry>,
        preset_handler: PresetHandler,
        updatable_handler: UpdatableHandler,
        timing_handler: TimingHandler,
        patch: Arc<ArcSwap<Patch>>,
    ) -> (Self, HashMap<u32, FixtureState>) {
        let fixture_state_handler = FixtureStateHandler::new(&patch.load()).unwrap();
        let fixture_states = fixture_state_handler.fixtures().clone();

        let state = DemexEngineState::default();

        let s = Self {
            event_bus_tx,
            request_handler,
            action_queue,
            value_queue_tx,
            preset_handler,
            updatable_handler,
            timing_handler,
            patch,

            fixture_state_handler,
            state,
        };

        (s, fixture_states)
    }
}

impl DemexThreadDelegate for UpdateThread {
    type ThreadMessage = ();

    fn name() -> &'static str {
        "demex-update"
    }

    fn its() -> f64 {
        60.0
    }

    fn update(&mut self, thread: &mut super::DemexThread<Self>) -> bool {
        for _ in thread.handle_messages() {}
        if thread.should_stop() {
            return true;
        }

        let patch = self.patch.load();
        let mut action_queue = self.action_queue.lock_write();

        // Handle queued actions
        // FIXME: just for testing
        for action in action_queue.inner_mut().drain(..) {
            match action.run(
                &mut self.fixture_state_handler,
                &mut self.preset_handler,
                FixtureSelectorContext::new(&self.state.fixture_selection),
                &mut self.updatable_handler,
                &mut DemexInputDeviceHandler::new(vec![]),
                &mut self.timing_handler,
                &patch,
            ) {
                Ok(result) => {
                    if action.issuer != ActionIssuer::Ui {
                        log::debug!("Action run result: {:?}", result);
                    }

                    let (result, event) = result.get_event();
                    if let Some(event) = event {
                        let _ = self
                            .event_bus_tx
                            .send(DemexEngineCommEvent::DemexEvent(event));
                    }

                    // Also send the result itself (maybe it should trigger a ui action)
                    let _ = self
                        .event_bus_tx
                        .send(DemexEngineCommEvent::ActionRunResult(result.clone()));

                    match result {
                        ActionRunResult::UpdateFixtureSelection(selection) => {
                            self.state.fixture_selection = selection.clone();
                            let _ = self.event_bus_tx.send(DemexEngineCommEvent::DemexEvent(
                                DemexEvent::FixtureSelectionChanged(selection),
                            ));
                        }
                        ActionRunResult::UpdatePatch(patch) => {
                            self.patch.store(Arc::new(patch));
                        }
                        ActionRunResult::WithEvent { .. } => unreachable!(),
                        _ => {}
                    }
                }
                Err(err) => {
                    let _ = self
                        .event_bus_tx
                        .send(DemexEngineCommEvent::Error(err.to_string()));
                    log::warn!("Failed to run action: {}", err);
                }
            }
        }

        self.timing_handler.update_running_timecodes(
            &mut self.fixture_state_handler,
            &self.preset_handler,
            &mut self.updatable_handler,
        );

        let mut updated_output_values = HashMap::new();
        let _ = self
            .fixture_state_handler
            .update_output_values(
                &patch,
                &self.preset_handler,
                &self.updatable_handler,
                &self.timing_handler,
                &mut updated_output_values,
            )
            .inspect_err(|err| log::error!("Failed to update fixture handler: {}", err));

        let _ = self
            .fixture_state_handler
            .submit_output_values(
                &self.value_queue_tx,
                &patch,
                &self.preset_handler,
                &self.timing_handler,
            )
            .inspect_err(|err| log::error!("Failed to submit output values: {}", err));

        let _uh_events = self.updatable_handler.update_executors(
            &patch,
            &mut self.fixture_state_handler,
            &self.preset_handler,
            &self.timing_handler,
        );

        // TODO: move the input device handler to the frontend
        /*
        input_device_event_handler.write(|handler| {
            handler.push_events(uh_events.into_iter().map(DemexEvent::ExecutorStop))
        });*/

        if !updated_output_values.is_empty() {
            let _ = self
                .event_bus_tx
                .send(DemexEngineCommEvent::FixtureValuesUpdate(
                    updated_output_values,
                ));
        }

        // handle ui requests
        let handler_payload = DemexEngineCommRequestHandlerPayload {
            patch: &patch,
            stats: thread.stats(),
            show: DemexShowRef {
                preset_handler: &self.preset_handler,
                updatable_handler: &self.updatable_handler,
                timing_handler: &self.timing_handler,
                input_device_configs: &vec![],
                patch: &patch,
            },
        };
        self.request_handler.handle_all(handler_payload);

        // send tick state
        // let tick_state = DemexEngineTickState {};
        // let _ = event_bus_tx.send(DemexEngineCommEvent::TickStateUpdate(tick_state));

        false
    }
}
