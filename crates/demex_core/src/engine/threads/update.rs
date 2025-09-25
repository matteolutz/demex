use crate::{
    engine::{component::ComponentHandle, threads::DEMEX_MAX_FUPS},
    fixture::handler::FixtureHandler,
    input::event::{DemexInputDeviceEvent, handler::DemexInputDeviceEventHandler},
    patch::Patch,
    presets::PresetHandler,
    timing::TimingHandler,
    updatables::UpdatableHandler,
    utils::thread::{DemexThreadStatsHandler, demex_update_thread},
};

pub fn start_demex_update_thread(
    stats: ComponentHandle<DemexThreadStatsHandler>,
    fixture_handler: ComponentHandle<FixtureHandler>,
    preset_handler: ComponentHandle<PresetHandler>,
    updatable_handler: ComponentHandle<UpdatableHandler>,
    timing_handler: ComponentHandle<TimingHandler>,
    patch: ComponentHandle<Patch>,
    input_device_event_handler: ComponentHandle<DemexInputDeviceEventHandler>,
) {
    let _ = input_device_event_handler;
    demex_update_thread(
        "demex-update".to_owned(),
        stats.clone(),
        DEMEX_MAX_FUPS,
        move |_, _| {
            let mut fixture_handler = fixture_handler
                .mutex_mut()
                .lock()
                .downcast_mut::<FixtureHandler>()
                .unwrap();

            let preset_handler = preset_handler_thread_b.read();
            let mut updatable_handler = updatable_handler_thread_b.write();
            let mut timing_handler = timing_handler_thread_b.write();
            let patch = patch_thread_b.read();
            let mut input_device_event_handler = input_device_event_handler_thread_b.write();

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
                    if args.controller { Some(&udp_tx) } else { None },
                )
                .inspect_err(|err| log::error!("Failed to update fixture handler: {}", err));

            input_device_event_handler.push_events(
                updatable_handler
                    .update_executors(
                        patch.fixture_types(),
                        &mut fixture_handler,
                        &preset_handler,
                        &timing_handler,
                    )
                    .into_iter()
                    .map(DemexInputDeviceEvent::ExecutorStop),
            );
        },
    );
}
