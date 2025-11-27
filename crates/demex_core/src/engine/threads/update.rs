use std::sync::mpsc;

use crate::{
    command::parser::nodes::{
        action::{ActionIssuer, queue::ActionQueue, result::ActionRunResult},
        fixture_selector::FixtureSelectorContext,
    },
    engine::{component::ComponentHandle, state::DemexEngineState, threads::DEMEX_MAX_FUPS},
    event::DemexEvent,
    fixture::handler::FixtureHandler,
    input::{DemexInputDeviceHandler, event::handler::DemexInputDeviceEventHandler},
    patch::Patch,
    presets::PresetHandler,
    timing::TimingHandler,
    updatables::UpdatableHandler,
    utils::thread::{DemexThreadStatsHandler, demex_update_thread},
};

pub fn start_demex_update_thread(
    event_bus_tx: mpsc::Sender<DemexEvent>,
    stats: ComponentHandle<DemexThreadStatsHandler>,
    action_queue: ComponentHandle<ActionQueue>,
    fixture_handler: ComponentHandle<FixtureHandler>,
    preset_handler: ComponentHandle<PresetHandler>,
    updatable_handler: ComponentHandle<UpdatableHandler>,
    timing_handler: ComponentHandle<TimingHandler>,
    patch: ComponentHandle<Patch>,
    mut input_device_event_handler: ComponentHandle<DemexInputDeviceEventHandler>,
    state: ComponentHandle<DemexEngineState>,
) {
    demex_update_thread(
        "demex-update".to_owned(),
        stats.clone(),
        DEMEX_MAX_FUPS,
        move |_, _| {
            let mut action_queue = action_queue.lock_write();

            let mut fixture_handler = fixture_handler.lock_write();

            let mut preset_handler = preset_handler.lock_write();

            let mut updatable_handler = updatable_handler.lock_write();

            let mut timing_handler = timing_handler.lock_write();

            let patch = patch.lock_read();

            let mut state = state.lock_write();

            // FIXME: just for testing
            for action in action_queue.inner_mut().drain(..) {
                match action.run(
                    &mut fixture_handler,
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
                &mut fixture_handler,
                &preset_handler,
                &mut updatable_handler,
            );

            let _ = fixture_handler
                .update_output_values(
                    patch.fixture_types(),
                    &preset_handler,
                    &updatable_handler,
                    &timing_handler,
                    /*if args.controller { Some(&udp_tx) } else { None },*/
                    None,
                )
                .inspect_err(|err| log::error!("Failed to update fixture handler: {}", err));

            let uh_events = updatable_handler.update_executors(
                patch.fixture_types(),
                &mut fixture_handler,
                &preset_handler,
                &timing_handler,
            );

            input_device_event_handler.write(|handler| {
                handler.push_events(uh_events.into_iter().map(DemexEvent::ExecutorStop))
            });
        },
    );
}
