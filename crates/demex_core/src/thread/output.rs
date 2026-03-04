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

/// The maximum output frames per second.
const MAX_OUTPUT_FPS: f64 = 44.0;

/// The minimum output frame time in seconds. Derived from [`MAX_OUTPUT_FPS`].
const MIN_OUTPUT_FRAME_TIME: f64 = 1.0 / MAX_OUTPUT_FPS;

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
        // we want to have the output thread run faster than the update thread
        // so we don't lag behind when receiving values from the value queue
        //
        // the acutal sending of values to the DMX outputs is done at MAX_OUTPUT_FPS
        MAX_OUTPUT_FPS * 2.0
    }

    fn update(&mut self, thread: &mut super::DemexThread<Self>) -> bool {
        for _ in thread.handle_messages() {}
        if thread.should_stop() {
            return true;
        }

        let patch = self.patch.load();
        let values = self.value_queue.try_iter().collect::<Vec<_>>();

        self.dmx_resolver.resovle(values, &patch);

        let elapsed = thread.last_user_update().elapsed().as_secs_f64();
        let should_output = elapsed > MIN_OUTPUT_FRAME_TIME;

        if should_output {
            let should_force = elapsed > 0.1;

            let updated_universes = self
                .dmx_resolver
                .send(self.outputs.iter_mut(), should_force);

            if updated_universes > 0 {
                *thread.last_user_update() = time::Instant::now();
            }
        }

        false
    }
}
