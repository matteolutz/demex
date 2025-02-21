use std::collections::HashMap;

use egui_probe::EguiProbe;
use serde::{Deserialize, Serialize};

use crate::fixture::{
    channel2::{channel_type::FixtureChannelType, channel_value::FixtureChannelValue2},
    presets::{preset::FixturePresetId, PresetHandler},
    selection::FixtureSelection,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default, EguiProbe)]
pub enum CueTrigger {
    /// Cue is triggered manually
    #[default]
    Manual,

    /// Cue is automatically triggered, after previous cue finished
    /// all of it's fading and delays
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

#[derive(Debug, Clone, Serialize, Deserialize, EguiProbe, Default)]
pub struct CueBuilderEntry {
    pub group_id: Option<u32>,
    pub preset_feature_group_id: Option<u32>,
    pub preset_id: Option<FixturePresetId>,
}

#[derive(Debug, Clone, Serialize, Deserialize, EguiProbe)]
pub enum CueDataMode {
    /// Default mode, where the data is stored as a map of fixture_id -> Vec<channel_values>
    Default(HashMap<u32, Vec<CueFixtureChannelValue>>),

    /// Builder mode (like MA 3 recipes), where the data is stored as a list of entries
    /// that are used to build the data. Each entry has a group_id or a preset_id.
    Builder(Vec<CueBuilderEntry>),
}

impl Default for CueDataMode {
    fn default() -> Self {
        Self::Default(HashMap::new())
    }
}

pub type CueIdx = (u32, u32);

#[derive(Debug, Clone, Serialize, Deserialize, Default, EguiProbe)]
pub struct Cue {
    #[egui_probe(skip)]
    cue_idx: CueIdx,

    #[serde(default)]
    name: String,

    #[egui_probe(skip)]
    // data: HashMap<u32, Vec<CueFixtureChannelValue>>,
    data: CueDataMode,

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
    pub fn new_default_builder(cue_idx: CueIdx) -> Self {
        Self {
            cue_idx,
            name: format!("Cue {}.{}", cue_idx.0, cue_idx.1),

            data: CueDataMode::Builder(Vec::new()),
            selection: FixtureSelection::default(),

            in_fade: 0.0,
            out_fade: None,
            in_delay: 0.0,
            out_delay: None,
            snap_percent: 0.0,
            timing: CueTiming::default(),
            trigger: CueTrigger::Manual,
        }
    }

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
            name: format!("Cue {}.{}", cue_idx.0, cue_idx.1),

            data: CueDataMode::Default(data),
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

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn name_mut(&mut self) -> &mut String {
        &mut self.name
    }

    pub fn data(&self) -> &CueDataMode {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut CueDataMode {
        &mut self.data
    }

    pub fn in_fade(&self) -> f32 {
        self.in_fade
    }

    pub fn in_fade_mut(&mut self) -> &mut f32 {
        &mut self.in_fade
    }

    pub fn out_fade(&self) -> f32 {
        self.out_fade.unwrap_or(self.in_fade)
    }

    pub fn out_fade_mut(&mut self) -> &mut Option<f32> {
        &mut self.out_fade
    }

    pub fn in_delay(&self) -> f32 {
        self.in_delay
    }

    pub fn in_delay_mut(&mut self) -> &mut f32 {
        &mut self.in_delay
    }

    pub fn out_delay(&self) -> f32 {
        self.out_delay.unwrap_or(self.in_delay)
    }

    pub fn out_delay_mut(&mut self) -> &mut Option<f32> {
        &mut self.out_delay
    }

    pub fn snap_percent(&self) -> f32 {
        self.snap_percent
    }

    pub fn snap_percent_mut(&mut self) -> &mut f32 {
        &mut self.snap_percent
    }

    pub fn timing(&self) -> &CueTiming {
        &self.timing
    }

    pub fn timing_mut(&mut self) -> &mut CueTiming {
        &mut self.timing
    }

    pub fn trigger(&self) -> &CueTrigger {
        &self.trigger
    }

    pub fn trigger_mut(&mut self) -> &mut CueTrigger {
        &mut self.trigger
    }

    pub fn total_offset(&self) -> f32 {
        self.timing.total_offset(self.selection.num_offsets())
    }

    pub fn offset_for_fixture(&self, fixture_id: u32) -> f32 {
        self.timing.offset_for_fixture(
            // TOOD: is .unwrap_or(0) the right thing to do?
            self.selection.offset_idx(fixture_id).unwrap_or(0),
            self.selection.num_offsets(),
        )
    }

    pub fn channel_value_for_fixture(
        &self,
        fixture_id: u32,
        channel_type: FixtureChannelType,
        preset_handler: &PresetHandler,
    ) -> Option<FixtureChannelValue2> {
        match &self.data {
            CueDataMode::Default(data) => data.get(&fixture_id).and_then(|values| {
                values
                    .iter()
                    .find(|v| v.channel_type() == channel_type)
                    .map(|v| v.value().clone())
            }),
            CueDataMode::Builder(entries) => {
                for entry in entries {
                    // if it's an empty entry, skip it
                    if entry.group_id.is_none() || entry.preset_id.is_none() {
                        continue;
                    }

                    let group = preset_handler.get_group(entry.group_id.unwrap());
                    if let Ok(group) = group {
                        // if the group doesn't have the fixture, skip it
                        if !group.fixture_selection().has_fixture(fixture_id) {
                            continue;
                        }

                        let preset = preset_handler.get_preset(entry.preset_id.unwrap());
                        if let Ok(preset) = preset {
                            if let Some(value) = preset.value(fixture_id, channel_type) {
                                // we found the value
                                return Some(FixtureChannelValue2::Discrete(value));
                            }
                        }
                    }
                }

                None
            }
        }
    }

    pub fn should_snap_channel_value_for_fixture(
        &self,
        fixture_id: u32,
        channel_type: FixtureChannelType,
    ) -> bool {
        match &self.data {
            CueDataMode::Default(data) => data
                .get(&fixture_id)
                .and_then(|values| {
                    values
                        .iter()
                        .find(|v| v.channel_type() == channel_type)
                        .map(|v| v.snap())
                })
                .unwrap_or(false),
            CueDataMode::Builder(_) => false,
        }
    }

    pub fn in_time(&self) -> f32 {
        self.in_delay + self.in_fade + self.total_offset()
    }

    pub fn out_time(&self) -> f32 {
        self.out_delay() + self.out_fade() + self.total_offset()
    }
}
