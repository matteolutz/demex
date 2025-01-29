use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::fixture::channel::{value::FixtureChannelValue, FixtureId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CueTrigger {
    // Cue is triggered manually
    Manual,

    // Cue is automatically triggered, after previous cue finished
    // all of it's fading and delays
    Follow,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CueFixtureChannelValue {
    value: FixtureChannelValue,
    channel_type: u16,
    snap: bool,
}

impl CueFixtureChannelValue {
    pub fn new(value: FixtureChannelValue, channel_type: u16, snap: bool) -> Self {
        Self {
            value,
            channel_type,
            snap,
        }
    }

    pub fn value(&self) -> &FixtureChannelValue {
        &self.value
    }

    pub fn channel_type(&self) -> u16 {
        self.channel_type
    }

    pub fn snap(&self) -> bool {
        self.snap
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cue {
    data: HashMap<FixtureId, Vec<CueFixtureChannelValue>>,

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
        data: HashMap<FixtureId, Vec<CueFixtureChannelValue>>,
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

    pub fn data(&self) -> &HashMap<FixtureId, Vec<CueFixtureChannelValue>> {
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

    pub fn data_for_fixture(&self, fixture_id: FixtureId) -> Option<&Vec<CueFixtureChannelValue>> {
        self.data.get(&fixture_id)
    }

    pub fn channel_value_for_fixture(
        &self,
        fixture_id: u32,
        channel_id: u16,
    ) -> Option<&FixtureChannelValue> {
        self.data.get(&fixture_id).and_then(|values| {
            values
                .iter()
                .find(|v| v.channel_type() == channel_id)
                .map(|v| v.value())
        })
    }

    pub fn should_snap_channel_value_for_fixture(&self, fixture_id: u32, channel_id: u16) -> bool {
        self.data
            .get(&fixture_id)
            .and_then(|values| {
                values
                    .iter()
                    .find(|v| v.channel_type() == channel_id)
                    .map(|v| v.snap())
            })
            .unwrap_or(false)
    }

    pub fn in_time(&self) -> f32 {
        self.in_delay + self.in_fade
    }

    pub fn out_time(&self) -> f32 {
        self.out_delay() + self.out_fade()
    }
}
