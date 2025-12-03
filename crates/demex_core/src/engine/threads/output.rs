use std::{
    sync::{Arc, mpsc},
    thread::JoinHandle,
    time,
};

use arc_swap::ArcSwap;
use demex_dmx::DemexDmxOutput;

use crate::{
    channel3::channel_value_queue::ChannelValueQueueEntry,
    dmx::dmx_resolver::DmxResolver,
    engine::{component::ComponentHandle, threads::DEMEX_MAX_OUTPUT_FUPS},
    patch::Patch,
    utils::thread::{DemexThreadStatsHandler, demex_update_thread},
};

pub fn start_demex_output_thread(
    stats: ComponentHandle<DemexThreadStatsHandler>,
    patch: Arc<ArcSwap<Patch>>,
    value_queue: mpsc::Receiver<ChannelValueQueueEntry>,
) -> JoinHandle<()> {
    let mut dmx_resolver = DmxResolver::default();

    let mut outputs = patch
        .load()
        .output_configs()
        .iter()
        .map(|config| {
            DemexDmxOutput::from_config(
                config.clone(),
                demex_headless::id::DemexProtoDeviceId::Controller,
            )
        })
        .collect::<Vec<_>>();

    demex_update_thread(
        "demex-dmx-output".to_owned(),
        stats.clone(),
        DEMEX_MAX_OUTPUT_FUPS,
        move |_, last_user_update| {
            let patch = patch.load();
            let values = value_queue.try_iter().collect::<Vec<_>>();

            dmx_resolver.resovle(values, &patch);

            let updated_universes = dmx_resolver.send(
                outputs.iter_mut(),
                last_user_update.elapsed().as_secs_f64() > 0.1,
            );

            if updated_universes > 0 {
                *last_user_update = time::Instant::now();
            }
        },
    )
}
