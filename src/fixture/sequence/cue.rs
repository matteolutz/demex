use std::collections::HashMap;

use crate::fixture::channel::{FixtureChannel, FixtureId};

#[derive(Debug, Clone)]
pub enum CueTrigger {
    // Cue is triggered manually
    Manual,

    // Cue is automatically triggered, after previous cue finished
    // all of it's fading and delays
    Follow,
}

#[derive(Debug, Clone)]
pub struct CueFixtureChannel {
    channel: FixtureChannel,
    snap: bool,
}

impl From<FixtureChannel> for CueFixtureChannel {
    fn from(channel: FixtureChannel) -> Self {
        return Self {
            channel,
            snap: false,
        };
    }
}

impl CueFixtureChannel {
    pub fn new(channel: FixtureChannel, snap: bool) -> Self {
        Self { channel, snap }
    }

    pub fn channel(&self) -> &FixtureChannel {
        &self.channel
    }

    pub fn snap(&self) -> bool {
        self.snap
    }
}

#[derive(Debug, Clone)]
pub struct Cue {
    data: HashMap<FixtureId, Vec<CueFixtureChannel>>,

    // Time, to fade into the cue
    in_fade: f32,
    // Time, to fade out of the cue
    out_fade: Option<f32>,

    // Delay, before the cue starts fading in
    in_delay: f32,

    // Delay, before the cue starts fading out
    out_delay: Option<f32>,

    // When (as a percentage of the in_fade time), snapping of values, that are not
    // being faded, are changed.
    snap_percent: f32,

    trigger: CueTrigger,
}

impl Cue {
    pub fn new(
        data: HashMap<FixtureId, Vec<CueFixtureChannel>>,
        in_fade: f32,
        out_fade: Option<f32>,
        in_delay: f32,
        out_delay: Option<f32>,
        snap_percent: f32,
        trigger: CueTrigger,
    ) -> Self {
        Self {
            data,
            in_fade,
            out_fade,
            in_delay,
            out_delay,
            snap_percent,
            trigger,
        }
    }

    pub fn data(&self) -> &HashMap<FixtureId, Vec<CueFixtureChannel>> {
        &self.data
    }

    pub fn in_fade(&self) -> f32 {
        self.in_fade
    }

    pub fn out_fade(&self) -> f32 {
        self.out_fade.unwrap_or(self.in_fade)
    }

    pub fn in_delay(&self) -> f32 {
        self.in_delay
    }

    pub fn out_delay(&self) -> f32 {
        self.out_delay.unwrap_or(self.in_delay)
    }

    pub fn snap_percent(&self) -> f32 {
        self.snap_percent
    }

    pub fn trigger(&self) -> &CueTrigger {
        &self.trigger
    }

    pub fn data_for_fixture(&self, fixture_id: FixtureId) -> Option<&Vec<CueFixtureChannel>> {
        self.data.get(&fixture_id)
    }
}
