use std::{
    collections::HashMap,
    sync::{Arc, mpsc},
};

use arc_swap::ArcSwap;

use crate::{
    channel3::channel_value_queue::ChannelValueQueueEntry,
    command::parser::nodes::{
        action::{
            ActionIssuer, DeferredActionRunArgs, queue::ActionQueue, result::ActionRunResult,
        },
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
    event::{DemexEvent, list::DemexEventList},
    fixture::FixturePath,
    input::DemexInputDeviceHandler,
    patch::Patch,
    pool::{PoolItem, PoolType},
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

    event_list: DemexEventList,

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
    ) -> (
        Self,
        HashMap<FixturePath, FixtureState>,
        HashMap<PoolType, Vec<PoolItem>>,
    ) {
        let fixture_state_handler = FixtureStateHandler::new(patch.load().fixtures()).unwrap();
        let fixture_states = fixture_state_handler.fixtures().clone();

        let show = DemexShowRef {
            preset_handler: &preset_handler,
            updatable_handler: &updatable_handler,
            timing_handler: &timing_handler,
            input_device_configs: &vec![],
            patch: &patch.load(),
        };
        let pools = PoolType::all()
            .into_iter()
            .filter_map(|pool_type| {
                show.get_pool(pool_type)
                    .get_all(pool_type)
                    .ok()
                    .map(|items| (pool_type, items))
            })
            .collect::<HashMap<_, _>>();

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

            event_list: DemexEventList::default(),

            fixture_state_handler,
            state,
        };

        (s, fixture_states, pools)
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
        // TODO: maybe limit amount of actions per frame
        for action in action_queue.inner_mut().drain(..) {
            let args = DeferredActionRunArgs {
                fixture_handler: &mut self.fixture_state_handler,
                preset_handler: &mut self.preset_handler,
                fixture_selector_context: FixtureSelectorContext::new(
                    &self.state.fixture_selection,
                ),
                updatable_handler: &mut self.updatable_handler,
                input_device_handler: &mut DemexInputDeviceHandler::new(vec![]),
                timing_handler: &mut self.timing_handler,
                patch: &patch,
                event_list: &mut self.event_list,
            };

            match action.run(args) {
                Ok(result) => {
                    if action.issuer != ActionIssuer::Ui {
                        log::debug!("Action run result: {:?}", result);
                    }

                    let _ = self
                        .event_bus_tx
                        .send(DemexEngineCommEvent::ActionRunResult(result.clone()));

                    match result {
                        ActionRunResult::UpdateFixtureSelection(selection) => {
                            self.state.fixture_selection =
                                selection.clone().map(|sel| sel.selection);
                            let _ = self.event_bus_tx.send(DemexEngineCommEvent::DemexEvent(
                                DemexEvent::FixtureSelectionChanged(selection),
                            ));
                        }
                        ActionRunResult::UpdateHighlight(highlight) => {
                            self.state.highlight = highlight.clone().map(|sel| sel.selection);
                            let _ = self.event_bus_tx.send(DemexEngineCommEvent::DemexEvent(
                                DemexEvent::HighlightChanged(highlight),
                            ));
                        }
                        ActionRunResult::UpdatePatch(patch) => {
                            self.patch.store(Arc::new(patch));
                        }
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
            &mut self.event_list,
        );

        let mut updated_output_values = HashMap::new();
        let _ = self
            .fixture_state_handler
            .update_output_values(
                &patch,
                &self.preset_handler,
                &self.updatable_handler,
                &self.timing_handler,
                self.state.highlight.as_ref(),
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

        self.updatable_handler.update_executors(
            &patch,
            &mut self.fixture_state_handler,
            &self.preset_handler,
            &self.timing_handler,
            &mut self.event_list,
        );
        // TODO: move the input device handler to the frontend
        /*
        input_device_event_handler.write(|handler| {
            handler.push_events(uh_events.into_iter().map(DemexEvent::ExecutorStop))
        });*/

        let _ = self.event_list.send(&self.event_bus_tx);
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
            state: &self.state,
        };
        self.request_handler.handle_all(handler_payload);

        // send tick state
        // let tick_state = DemexEngineTickState {};
        // let _ = event_bus_tx.send(DemexEngineCommEvent::TickStateUpdate(tick_state));

        false
    }
}
