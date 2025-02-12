use std::collections::HashMap;

use egui_probe::EguiProbe;
use serde::{Deserialize, Serialize};

use crate::fixture::channel::{
    FIXTURE_CHANNEL_COLOR_ID, FIXTURE_CHANNEL_INTENSITY_ID, FIXTURE_CHANNEL_POSITION_PAN_TILT_ID,
    FIXTURE_CHANNEL_SHUTTER_ID, FIXTURE_CHANNEL_TOGGLE_FLAGS, FIXTURE_CHANNEL_ZOOM_ID,
};

pub const DEFAULT_FEATURE_GROUP_INTENSITY_ID: u32 = 0;
pub const DEFAULT_FEATURE_GROUP_POSITION_ID: u32 = 1;
pub const DEFAULT_FEATURE_GROUP_COLOR_ID: u32 = 2;
pub const DEFAULT_FEATURE_GROUP_BEAM_ID: u32 = 3;
pub const DEFAULT_FEATURE_GROUP_FOCUS_ID: u32 = 4;
pub const DEFAULT_FEATURE_GROUP_CONTROL_ID: u32 = 5;

#[derive(Debug, Serialize, Deserialize, Default, EguiProbe, Clone)]
pub struct FeatureGroup {
    id: u32,
    name: String,
    channel_types: Vec<u16>,
}

impl FeatureGroup {
    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn name_mut(&mut self) -> &mut String {
        &mut self.name
    }

    pub fn channel_types(&self) -> &[u16] {
        &self.channel_types
    }
}

impl FeatureGroup {
    pub fn default_feature_groups() -> HashMap<u32, FeatureGroup> {
        let mut feature_groups = HashMap::new();

        // intensity
        feature_groups.insert(
            DEFAULT_FEATURE_GROUP_INTENSITY_ID,
            FeatureGroup {
                id: DEFAULT_FEATURE_GROUP_INTENSITY_ID,
                name: "Intensity".to_owned(),
                channel_types: vec![FIXTURE_CHANNEL_INTENSITY_ID],
            },
        );

        // position
        feature_groups.insert(
            DEFAULT_FEATURE_GROUP_POSITION_ID,
            FeatureGroup {
                id: DEFAULT_FEATURE_GROUP_POSITION_ID,
                name: "Position".to_owned(),
                channel_types: vec![FIXTURE_CHANNEL_POSITION_PAN_TILT_ID],
            },
        );

        // color
        feature_groups.insert(
            DEFAULT_FEATURE_GROUP_COLOR_ID,
            FeatureGroup {
                id: DEFAULT_FEATURE_GROUP_COLOR_ID,
                name: "Color".to_owned(),
                channel_types: vec![FIXTURE_CHANNEL_COLOR_ID],
            },
        );

        // beam
        feature_groups.insert(
            DEFAULT_FEATURE_GROUP_BEAM_ID,
            FeatureGroup {
                id: DEFAULT_FEATURE_GROUP_BEAM_ID,
                name: "Beam".to_owned(),
                channel_types: vec![FIXTURE_CHANNEL_SHUTTER_ID],
            },
        );

        // focus
        feature_groups.insert(
            DEFAULT_FEATURE_GROUP_FOCUS_ID,
            FeatureGroup {
                id: DEFAULT_FEATURE_GROUP_FOCUS_ID,
                name: "Focus".to_owned(),
                channel_types: vec![FIXTURE_CHANNEL_ZOOM_ID],
            },
        );

        // control
        feature_groups.insert(
            DEFAULT_FEATURE_GROUP_CONTROL_ID,
            FeatureGroup {
                id: DEFAULT_FEATURE_GROUP_CONTROL_ID,
                name: "Control".to_owned(),
                channel_types: vec![FIXTURE_CHANNEL_TOGGLE_FLAGS],
            },
        );

        feature_groups
    }
}
