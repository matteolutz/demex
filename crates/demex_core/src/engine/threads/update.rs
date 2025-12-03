use std::{
    collections::HashMap,
    sync::{Arc, mpsc},
    thread::JoinHandle,
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
        threads::DEMEX_MAX_FUPS,
    },
    event::DemexEvent,
    input::DemexInputDeviceHandler,
    patch::Patch,
    presets::PresetHandler,
    state::{fixture_state::FixtureState, fixture_state_handler::FixtureStateHandler},
    timing::TimingHandler,
    updatables::UpdatableHandler,
    utils::thread::{DemexThreadStatsHandler, demex_update_thread},
};

pub(crate) fn start_demex_update_thread(
    event_bus_tx: mpsc::Sender<DemexEngineCommEvent>,
    request_handler: DemexEngineCommRequestHandler,
    stats: ComponentHandle<DemexThreadStatsHandler>,
    action_queue: ComponentHandle<ActionQueue>,
    value_queue_tx: mpsc::Sender<ChannelValueQueueEntry>,
    mut preset_handler: PresetHandler,
    mut updatable_handler: UpdatableHandler,
    mut timing_handler: TimingHandler,
    patch_swap: Arc<ArcSwap<Patch>>,
) -> (JoinHandle<()>, HashMap<u32, FixtureState>) {
    let mut fixture_state_handler = FixtureStateHandler::new(&patch_swap.load()).unwrap();
    let fixture_states = fixture_state_handler.fixtures().clone();

    let mut state = DemexEngineState::default();

    let jh = demex_update_thread(
        "demex-update".to_owned(),
        stats.clone(),
        DEMEX_MAX_FUPS,
        move |_, _| {
            let patch = patch_swap.load();
            let mut action_queue = action_queue.lock_write();

            // Handle queued actions
            // FIXME: just for testing
            for action in action_queue.inner_mut().drain(..) {
                match action.run(
                    &mut fixture_state_handler,
                    &mut preset_handler,
                    FixtureSelectorContext::new(&state.fixture_selection),
                    &mut updatable_handler,
                    &mut DemexInputDeviceHandler::new(vec![]),
                    &mut timing_handler,
                    &patch,
                ) {
                    Ok(result) => {
                        if action.issuer != ActionIssuer::Ui {
                            log::debug!("Action run result: {:?}", result);
                        }

                        let (result, event) = result.get_event();
                        if let Some(event) = event {
                            let _ = event_bus_tx.send(DemexEngineCommEvent::DemexEvent(event));
                        }

                        // Also send the result itself (maybe it should trigger a ui action)
                        let _ = event_bus_tx
                            .send(DemexEngineCommEvent::ActionRunResult(result.clone()));

                        match result {
                            ActionRunResult::UpdateFixtureSelection(selection) => {
                                state.fixture_selection = selection.clone();
                                let _ = event_bus_tx.send(DemexEngineCommEvent::DemexEvent(
                                    DemexEvent::FixtureSelectionChanged(selection),
                                ));
                            }
                            ActionRunResult::UpdatePatch(patch) => {
                                patch_swap.store(Arc::new(patch));
                            }
                            ActionRunResult::WithEvent { .. } => unreachable!(),
                            _ => {}
                        }
                    }
                    Err(err) => {
                        let _ = event_bus_tx.send(DemexEngineCommEvent::Error(err.to_string()));
                        log::warn!("Failed to run action: {}", err);
                    }
                }
            }

            timing_handler.update_running_timecodes(
                &mut fixture_state_handler,
                &preset_handler,
                &mut updatable_handler,
            );

            let mut updated_output_values = HashMap::new();
            let _ = fixture_state_handler
                .update_output_values(
                    &patch,
                    &preset_handler,
                    &updatable_handler,
                    &timing_handler,
                    &mut updated_output_values,
                )
                .inspect_err(|err| log::error!("Failed to update fixture handler: {}", err));

            let _ = fixture_state_handler
                .submit_output_values(&value_queue_tx, &patch, &preset_handler, &timing_handler)
                .inspect_err(|err| log::error!("Failed to submit output values: {}", err));

            let _uh_events = updatable_handler.update_executors(
                &patch,
                &mut fixture_state_handler,
                &preset_handler,
                &timing_handler,
            );

            // TODO: move the input device handler to the frontend
            /*
            input_device_event_handler.write(|handler| {
                handler.push_events(uh_events.into_iter().map(DemexEvent::ExecutorStop))
            });*/

            if !updated_output_values.is_empty() {
                let _ = event_bus_tx.send(DemexEngineCommEvent::FixtureValuesUpdate(
                    updated_output_values,
                ));
            }

            // handle ui requests
            let handler_payload = DemexEngineCommRequestHandlerPayload {
                patch: &patch,
                stats: &stats,
            };
            request_handler.handle_all(handler_payload);

            // send tick state
            // let tick_state = DemexEngineTickState {};
            // let _ = event_bus_tx.send(DemexEngineCommEvent::TickStateUpdate(tick_state));
        },
    );

    (jh, fixture_states)
}
