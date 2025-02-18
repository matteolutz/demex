use std::collections::HashMap;

use egui_probe::EguiProbe;
use serde::{Deserialize, Serialize};

use crate::fixture::{
    channel2::{channel_type::FixtureChannelType, channel_value::FixtureChannelValue2},
    selection::FixtureSelection,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default, EguiProbe)]
pub enum CueTrigger {
    // Cue is triggered manually
    #[default]
    Manual,

    // Cue is automatically triggered, after previous cue finished
    // all of it's fading and delays
    Follow,
}

#[derive(Debug, Clone, Serialize, Deserialize, EguiProbe, Default)]
pub struct CueFixtureChannelValue {
    value: FixtureChannelValue2,
    channel_type: FixtureChannelType,
    snap: bool,
}

impl CueFixtureChannelValue {
    pub fn new(value: FixtureChannelValue2, channel_type: FixtureChannelType, snap: bool) -> Self {
        Self {
            value,
            channel_type,
            snap,
        }
    }

    pub fn value(&self) -> &FixtureChannelValue2 {
        &self.value
    }

    pub fn channel_type(&self) -> FixtureChannelType {
        self.channel_type
    }

    pub fn snap(&self) -> bool {
        self.snap
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Default, EguiProbe)]
pub enum CueTimingOriginDirection {
    #[default]
    LowToHigh,
    HighToLow,
    CenterToOutside,
    OutsideToCenter,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, EguiProbe)]
pub struct CueTiming {
    // Offset (in seconds), that is applied between the fade in and down
    // of each fixture
    offset: f32,

    // Oirgin, where the offset is applied
    direction: CueTimingOriginDirection,
}

impl CueTiming {
    pub fn offset(&self) -> f32 {
        self.offset
    }

    pub fn origin(&self) -> CueTimingOriginDirection {
        self.direction
    }

    pub fn total_offset(&self, num_offsets: usize) -> f32 {
        let offset = match self.direction {
            CueTimingOriginDirection::LowToHigh | CueTimingOriginDirection::HighToLow => {
                self.offset * (num_offsets as f32 - 1.0)
            }
            CueTimingOriginDirection::CenterToOutside
            | CueTimingOriginDirection::OutsideToCenter => {
                let half_fixtures = f32::ceil(num_offsets as f32 / 2.0);

                self.offset * (half_fixtures - 1.0)
            }
        };

        f32::max(offset, 0.0)
    }

    pub fn offset_for_fixture(&self, fixture_offset_idx: usize, num_fixtures: usize) -> f32 {
        match self.direction {
            CueTimingOriginDirection::LowToHigh => self.offset * fixture_offset_idx as f32,
            CueTimingOriginDirection::HighToLow => {
                self.offset * (num_fixtures as f32 - 1.0 - fixture_offset_idx as f32)
            }
            CueTimingOriginDirection::CenterToOutside => {
                let center = f32::max((num_fixtures as f32 / 2.0) + 0.5, 0.0);
                let center_offset = f32::floor(f32::abs(center - (fixture_offset_idx + 1) as f32));

                self.offset * center_offset
            }
            CueTimingOriginDirection::OutsideToCenter => {
                let center = f32::max((num_fixtures as f32 / 2.0) + 0.5, 0.0);
                let center_offset = f32::floor(f32::abs(center - (fixture_offset_idx + 1) as f32));

                self.offset * ((num_fixtures / 2) as f32 - 1.0 - center_offset)
            }
        }
    }
}

pub type CueIdx = (u32, u32);

#[derive(Debug, Clone, Serialize, Deserialize, Default, EguiProbe)]
pub struct Cue {
    #[egui_probe(skip)]
    cue_idx: CueIdx,

    #[egui_probe(skip)]
    data: HashMap<u32, Vec<CueFixtureChannelValue>>,

    selection: FixtureSelection,

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

    #[serde(default)]
    timing: CueTiming,

    trigger: CueTrigger,
}

impl Cue {
    pub fn new(
        cue_idx: CueIdx,
        data: HashMap<u32, Vec<CueFixtureChannelValue>>,
        selection: FixtureSelection,
        in_fade: f32,
        out_fade: Option<f32>,
        in_delay: f32,
        out_delay: Option<f32>,
        snap_percent: f32,
        timing: CueTiming,
        trigger: CueTrigger,
    ) -> Self {
        Self {
            cue_idx,

            data,
            selection,

            in_fade,
            out_fade,
            in_delay,
            out_delay,
            snap_percent,
            timing,
            trigger,
        }
    }

    pub fn cue_idx(&self) -> CueIdx {
        self.cue_idx
    }

    pub fn data(&self) -> &HashMap<u32, Vec<CueFixtureChannelValue>> {
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

    pub fn timing(&self) -> &CueTiming {
        &self.timing
    }

    pub fn trigger(&self) -> &CueTrigger {
        &self.trigger
    }

    pub fn num_fixtures(&self) -> usize {
        self.data.len()
    }

    pub fn total_offset(&self) -> f32 {
        self.timing.total_offset(self.selection.num_offsets())
    }

    pub fn offset_for_fixture(&self, fixture_id: u32) -> f32 {
        self.timing.offset_for_fixture(
            // TOOD: is .unwrap_or(0) the right thing to do?
            self.selection.offset_idx(fixture_id).unwrap_or(0),
            self.num_fixtures(),
        )
    }

    pub fn data_for_fixture(&self, fixture_id: u32) -> Option<&Vec<CueFixtureChannelValue>> {
        self.data.get(&fixture_id)
    }

    pub fn channel_value_for_fixture(
        &self,
        fixture_id: u32,
        channel_type: FixtureChannelType,
    ) -> Option<&FixtureChannelValue2> {
        self.data.get(&fixture_id).and_then(|values| {
            values
                .iter()
                .find(|v| v.channel_type() == channel_type)
                .map(|v| v.value())
        })
    }

    pub fn should_snap_channel_value_for_fixture(
        &self,
        fixture_id: u32,
        channel_type: FixtureChannelType,
    ) -> bool {
        self.data
            .get(&fixture_id)
            .and_then(|values| {
                values
                    .iter()
                    .find(|v| v.channel_type() == channel_type)
                    .map(|v| v.snap())
            })
            .unwrap_or(false)
    }

    pub fn in_time(&self) -> f32 {
        self.in_delay + self.in_fade + self.total_offset()
    }

    pub fn out_time(&self) -> f32 {
        self.out_delay() + self.out_fade() + self.total_offset()
    }
}
