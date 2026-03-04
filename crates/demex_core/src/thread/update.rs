use std::{
    collections::HashMap,
    sync::{Arc, mpsc},
    time::Instant,
};

use arc_swap::ArcSwap;

use crate::{
    channel3::channel_value_queue::ChannelValueQueueEntry,
    command::{
        lexer::Lexer,
        parser::{
            Parser2,
            nodes::{
                action::{
                    ActionIssuer, ActionRunArgs, DeferredActionRunArgs, queue::ActionQueue,
                    result::ActionRunResult,
                },
                fixture_selector::FixtureSelectorContext,
            },
        },
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
    input::{DemexInputDeviceHandler, device::DemexInputDeviceConfig},
    master::MasterHandler,
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
    master_handler: MasterHandler,
    patch: Arc<ArcSwap<Patch>>,

    command_input: Arc<ArcSwap<String>>,

    event_list: DemexEventList,

    input_device_handler: DemexInputDeviceHandler,

    fixture_state_handler: FixtureStateHandler,
    state: DemexEngineState,
}

impl UpdateThread {
    pub fn new(
        event_bus_tx: mpsc::Sender<DemexEngineCommEvent>,
        request_handler: DemexEngineCommRequestHandler,
        action_queue: ComponentHandle<ActionQueue>,
        value_queue_tx: mpsc::Sender<ChannelValueQueueEntry>,
        command_input: Arc<ArcSwap<String>>,
        mut preset_handler: PresetHandler,
        mut updatable_handler: UpdatableHandler,
        mut timing_handler: TimingHandler,
        input_device_configs: Vec<DemexInputDeviceConfig>,
        patch: Arc<ArcSwap<Patch>>,
    ) -> (
        Self,
        HashMap<FixturePath, FixtureState>,
        HashMap<PoolType, Vec<PoolItem>>,
    ) {
        let mut fixture_state_handler = FixtureStateHandler::new(patch.load().fixtures()).unwrap();
        let fixture_states = fixture_state_handler.fixtures().clone();

        let mut master_handler = MasterHandler::new(&preset_handler);

        // TODO: input device init state
        let args = ActionRunArgs {
            issued_at: Instant::now(),
            patch: &patch.load(),
            fixture_handler: &mut fixture_state_handler,
            preset_handler: &mut preset_handler,
            updatable_handler: &mut updatable_handler,
            timing_handler: &mut timing_handler,
            master_handler: &mut master_handler,
            fixture_selector_context: FixtureSelectorContext::new(&None),
            event_list: &mut DemexEventList::new(),
        };

        let input_devices = input_device_configs
            .into_iter()
            .filter_map(|config| {
                config
                    .into_device(&args)
                    .inspect_err(|err| log::error!("Failed to init device: {}", err))
                    .ok()
            })
            .collect();
        let input_device_handler = DemexInputDeviceHandler::new(input_devices);

        let show = DemexShowRef {
            preset_handler: &preset_handler,
            updatable_handler: &updatable_handler,
            timing_handler: &timing_handler,
            input_device_handler: &input_device_handler,
            master_handler: &master_handler,
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
            master_handler,
            patch,

            command_input,

            event_list: DemexEventList::default(),

            input_device_handler,

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

        // Handle queued actions
        // TODO: maybe limit amount of actions per frame
        for action in self.action_queue.lock_write().inner_mut().drain(..) {
            let args = DeferredActionRunArgs {
                fixture_handler: &mut self.fixture_state_handler,
                preset_handler: &mut self.preset_handler,
                fixture_selector_context: FixtureSelectorContext::new(
                    &self.state.fixture_selection,
                ),
                updatable_handler: &mut self.updatable_handler,
                timing_handler: &mut self.timing_handler,
                master_handler: &mut self.master_handler,
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
                        ActionRunResult::Assign(assignment) => {
                            if let Err(err) = self.input_device_handler.assign(assignment) {
                                let _ = self
                                    .event_bus_tx
                                    .send(DemexEngineCommEvent::Error(err.to_string()));
                            }
                        }
                        ActionRunResult::AssignMultiple(assignments) => {
                            for assignment in assignments {
                                if let Err(err) = self.input_device_handler.assign(assignment) {
                                    let _ = self
                                        .event_bus_tx
                                        .send(DemexEngineCommEvent::Error(err.to_string()));
                                }
                            }
                        }
                        ActionRunResult::Unassign(unassignment) => {
                            if let Err(err) = self.input_device_handler.unassign(unassignment) {
                                let _ = self
                                    .event_bus_tx
                                    .send(DemexEngineCommEvent::Error(err.to_string()));
                            }
                        }
                        ActionRunResult::GroupFixturesChanges(_)
                        | ActionRunResult::GroupAdded(_)
                        | ActionRunResult::GroupsRemoved(_) => {
                            self.master_handler.invalidate_cache(&self.preset_handler);
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
                &mut self.master_handler,
            )
            .inspect_err(|err| log::error!("Failed to submit output values: {}", err));

        self.updatable_handler.update_executors(
            &patch,
            &mut self.fixture_state_handler,
            &self.preset_handler,
            &self.timing_handler,
            &mut self.event_list,
        );

        let _ = self
            .input_device_handler
            .update(
                &patch,
                FixtureSelectorContext::new(&mut self.state.fixture_selection),
                &mut self.action_queue.lock_write(),
                |str_to_append| {
                    let _ = self
                        .event_bus_tx
                        .send(DemexEngineCommEvent::AppendToCommandInput(str_to_append));
                },
                || {
                    let command_input = self.command_input.load();
                    let tokens = Lexer::new(&command_input).tokenize().ok()?;
                    Parser2::new(&tokens).parse().err()
                },
                None,
                &mut self.event_list,
            )
            .inspect_err(|err| log::error!("Failed to update input device handler: {}", err));

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
                input_device_handler: &self.input_device_handler,
                master_handler: &self.master_handler,
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
