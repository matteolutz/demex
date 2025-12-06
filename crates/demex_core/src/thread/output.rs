use std::{
    sync::{Arc, mpsc},
    time,
};

use arc_swap::ArcSwap;
use demex_dmx::DemexDmxOutput;

use crate::{
    channel3::channel_value_queue::ChannelValueQueueEntry, dmx::dmx_resolver::DmxResolver,
    patch::Patch, thread::DemexThreadDelegate,
};

pub struct OutputThread {
    patch: Arc<ArcSwap<Patch>>,
    value_queue: mpsc::Receiver<ChannelValueQueueEntry>,

    dmx_resolver: DmxResolver,
    outputs: Vec<DemexDmxOutput>,
}

impl OutputThread {
    pub fn new(
        patch: Arc<ArcSwap<Patch>>,
        value_pipeline: mpsc::Receiver<ChannelValueQueueEntry>,
    ) -> Self {
        let dmx_resolver = DmxResolver::default();

        let outputs = patch
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

        Self {
            patch,
            value_queue: value_pipeline,
            dmx_resolver,
            outputs,
        }
    }
}

impl DemexThreadDelegate for OutputThread {
    type ThreadMessage = ();

    fn name() -> &'static str {
        "demex-output"
    }

    fn its() -> f64 {
        30.0
    }

    fn update(&mut self, thread: &mut super::DemexThread<Self>) -> bool {
        for _ in thread.handle_messages() {}
        if thread.should_stop() {
            return true;
        }

        let patch = self.patch.load();
        let values = self.value_queue.try_iter().collect::<Vec<_>>();

        self.dmx_resolver.resovle(values, &patch);

        let updated_universes = self.dmx_resolver.send(
            self.outputs.iter_mut(),
            thread.last_user_update().elapsed().as_secs_f64() > 0.1,
        );

        if updated_universes > 0 {
            *thread.last_user_update() = time::Instant::now();
        }

        false
    }
}
