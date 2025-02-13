use std::collections::HashMap;

use egui_probe::EguiProbe;
use serde::{Deserialize, Serialize};

use crate::fixture::channel2::channel_type::FixtureChannelType;

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
    channel_types: Vec<FixtureChannelType>,
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

    pub fn channel_types(&self) -> &[FixtureChannelType] {
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
                channel_types: vec![
                    FixtureChannelType::Intensity,
                    FixtureChannelType::IntensityFine,
                ],
            },
        );

        // position
        feature_groups.insert(
            DEFAULT_FEATURE_GROUP_POSITION_ID,
            FeatureGroup {
                id: DEFAULT_FEATURE_GROUP_POSITION_ID,
                name: "Position".to_owned(),
                channel_types: vec![
                    FixtureChannelType::Pan,
                    FixtureChannelType::PanFine,
                    FixtureChannelType::Tilt,
                    FixtureChannelType::TiltFine,
                ],
            },
        );

        // color
        feature_groups.insert(
            DEFAULT_FEATURE_GROUP_COLOR_ID,
            FeatureGroup {
                id: DEFAULT_FEATURE_GROUP_COLOR_ID,
                name: "Color".to_owned(),
                channel_types: vec![
                    FixtureChannelType::Red,
                    FixtureChannelType::RedFine,
                    FixtureChannelType::Green,
                    FixtureChannelType::GreenFine,
                    FixtureChannelType::Blue,
                    FixtureChannelType::BlueFine,
                ],
            },
        );

        // beam
        feature_groups.insert(
            DEFAULT_FEATURE_GROUP_BEAM_ID,
            FeatureGroup {
                id: DEFAULT_FEATURE_GROUP_BEAM_ID,
                name: "Beam".to_owned(),
                channel_types: vec![],
            },
        );

        // focus
        feature_groups.insert(
            DEFAULT_FEATURE_GROUP_FOCUS_ID,
            FeatureGroup {
                id: DEFAULT_FEATURE_GROUP_FOCUS_ID,
                name: "Focus".to_owned(),
                channel_types: vec![],
            },
        );

        // control
        feature_groups.insert(
            DEFAULT_FEATURE_GROUP_CONTROL_ID,
            FeatureGroup {
                id: DEFAULT_FEATURE_GROUP_CONTROL_ID,
                name: "Control".to_owned(),
                channel_types: vec![
                    FixtureChannelType::ToggleFlags(0),
                    FixtureChannelType::ToggleFlags(1),
                    FixtureChannelType::ToggleFlags(2),
                    FixtureChannelType::ToggleFlags(3),
                    FixtureChannelType::ToggleFlags(4),
                    FixtureChannelType::ToggleFlags(5),
                ],
            },
        );

        feature_groups
    }
}
