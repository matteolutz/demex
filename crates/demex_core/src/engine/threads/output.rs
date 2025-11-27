use std::time;

use crate::{
    engine::{component::ComponentHandle, threads::DEMEX_MAX_OUTPUT_FUPS},
    fixture::handler::FixtureHandler,
    patch::Patch,
    presets::PresetHandler,
    timing::TimingHandler,
    utils::thread::{DemexThreadStatsHandler, demex_update_thread},
};

pub fn start_demex_output_thread(
    stats: ComponentHandle<DemexThreadStatsHandler>,
    fixture_handler: ComponentHandle<FixtureHandler>,
    preset_handler: ComponentHandle<PresetHandler>,
    timing_handler: ComponentHandle<TimingHandler>,
    patch: ComponentHandle<Patch>,
) {
    demex_update_thread(
        "demex-dmx-output".to_owned(),
        stats.clone(),
        DEMEX_MAX_OUTPUT_FUPS,
        move |_, last_user_update| {
            let mut fixture_handler = fixture_handler.lock_write();
            let preset_handler = preset_handler.lock_read();
            let timing_handler = timing_handler.lock_read();
            let patch = patch.lock_read();

            if fixture_handler
                .generate_output_data(
                    patch.fixture_types(),
                    &preset_handler,
                    &timing_handler,
                    last_user_update.elapsed().as_secs_f64() > 0.1,
                )
                .inspect_err(|err| log::error!("Failed to generate output data: {}", err))
                .is_ok_and(|res| res > 0)
            {
                *last_user_update = time::Instant::now();
            }
        },
    );
}
