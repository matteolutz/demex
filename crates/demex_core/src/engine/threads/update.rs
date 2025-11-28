use std::{sync::mpsc, thread::JoinHandle};

use arc_swap::ArcSwap;

use crate::{
    channel3::channel_value_queue::ChannelValueQueueEntry,
    command::parser::nodes::{
        action::{ActionIssuer, queue::ActionQueue, result::ActionRunResult},
        fixture_selector::FixtureSelectorContext,
    },
    engine::{component::ComponentHandle, state::DemexEngineState, threads::DEMEX_MAX_FUPS},
    event::DemexEvent,
    input::DemexInputDeviceHandler,
    patch::Patch,
    presets::PresetHandler,
    state::fixture_state_handler::FixtureStateHandler,
    timing::TimingHandler,
    updatables::UpdatableHandler,
    utils::thread::{DemexThreadStatsHandler, demex_update_thread},
};

pub fn start_demex_update_thread(
    event_bus_tx: mpsc::Sender<DemexEvent>,
    stats: ComponentHandle<DemexThreadStatsHandler>,
    action_queue: ComponentHandle<ActionQueue>,
    value_queue_tx: mpsc::Sender<ChannelValueQueueEntry>,
    mut preset_handler: PresetHandler,
    mut updatable_handler: UpdatableHandler,
    mut timing_handler: TimingHandler,
    patch: &ArcSwap<Patch>,
) -> JoinHandle<()> {
    let patch = patch.load();
    let mut fixture_state_handler = FixtureStateHandler::new(&patch).unwrap();
    let mut state = DemexEngineState::default();

    demex_update_thread(
        "demex-update".to_owned(),
        stats.clone(),
        DEMEX_MAX_FUPS,
        move |_, _| {
            let mut action_queue = action_queue.lock_write();

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
                            let _ = event_bus_tx.send(event);
                        }

                        match result {
                            ActionRunResult::UpdateFixtureSelection(selection) => {
                                state.fixture_selection = selection.clone();
                                let _ = event_bus_tx
                                    .send(DemexEvent::FixtureSelectionChanged(selection));
                            }
                            ActionRunResult::WithEvent { .. } => unreachable!(),
                            _ => {}
                        }
                    }
                    Err(err) => log::warn!("Failed to run action: {}", err),
                }
            }

            timing_handler.update_running_timecodes(
                &mut fixture_state_handler,
                &preset_handler,
                &mut updatable_handler,
            );

            let _ = fixture_state_handler
                .update_output_values(&patch, &preset_handler, &updatable_handler, &timing_handler)
                .inspect_err(|err| log::error!("Failed to update fixture handler: {}", err));

            let _ = fixture_state_handler
                .submit_output_values(&value_queue_tx, &patch, &preset_handler, &timing_handler)
                .inspect_err(|err| log::error!("Failed to submit output values: {}", err));

            let uh_events = updatable_handler.update_executors(
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
        },
    )
}
