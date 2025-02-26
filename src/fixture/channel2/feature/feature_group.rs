use std::collections::HashMap;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use egui_probe::EguiProbe;
use serde::{Deserialize, Serialize};

use crate::fixture::{
    channel2::{
        channel_type::FixtureChannelType, channel_value::FixtureChannelValue2,
        error::FixtureChannelError2, feature::feature_type::FixtureFeatureType,
    },
    presets::PresetHandler,
};

use super::{feature_config::FixtureFeatureConfig, feature_state::FixtureFeatureDisplayState};

#[derive(Debug, Copy, Clone, PartialEq, Eq, EnumIter, Default)]
pub enum DefaultFeatureGroup {
    Intensity,
    Position,
    Color,
    Beam,
    Focus,
    Control,

    #[default]
    Unused,
}

impl DefaultFeatureGroup {
    pub fn id(&self) -> u32 {
        match self {
            Self::Intensity => 0,
            Self::Position => 1,
            Self::Color => 2,
            Self::Beam => 3,
            Self::Focus => 4,
            Self::Control => 5,
            Self::Unused => unreachable!(),
        }
    }

    pub fn feature_types(&self) -> Vec<FixtureFeatureType> {
        let single_value_types = FixtureChannelType::iter()
            .filter(|channel_type| {
                channel_type
                    .default_feature_group()
                    .is_some_and(|g| g == *self)
            })
            .map(|channel_type| FixtureFeatureType::SingleValue { channel_type });

        let feature_types = match self {
            Self::Position => vec![FixtureFeatureType::PositionPanTilt],
            Self::Color => vec![FixtureFeatureType::ColorRGB, FixtureFeatureType::ColorMacro],
            _ => vec![],
        };

        single_value_types
            .chain(feature_types.into_iter())
            .collect()
    }

    pub fn get_all() -> [DefaultFeatureGroup; 6] {
        [
            Self::Intensity,
            Self::Position,
            Self::Color,
            Self::Beam,
            Self::Focus,
            Self::Control,
        ]
    }
}

#[derive(Debug, Serialize, Deserialize, Default, EguiProbe, Clone)]
pub struct FeatureGroup {
    id: u32,
    name: String,
    feature_types: Vec<FixtureFeatureType>,
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

    pub fn feature_types(&self) -> &[FixtureFeatureType] {
        &self.feature_types
    }
}

impl FeatureGroup {
    pub fn default_feature_groups() -> HashMap<u32, FeatureGroup> {
        DefaultFeatureGroup::get_all()
            .into_iter()
            .map(|default_feature_group| {
                (
                    default_feature_group.id(),
                    FeatureGroup {
                        id: default_feature_group.id(),
                        name: format!("{:?}", default_feature_group),
                        feature_types: default_feature_group.feature_types(),
                    },
                )
            })
            .collect::<HashMap<_, _>>()
    }

    pub fn get_display_state(
        &self,
        fixture_id: u32,
        feature_configs: &[FixtureFeatureConfig],
        channels: &impl Fn(FixtureChannelType) -> Option<FixtureChannelValue2>,
        preset_handler: &PresetHandler,
    ) -> Result<Vec<FixtureFeatureDisplayState>, FixtureChannelError2> {
        Ok(self
            .feature_types
            .iter()
            .map(|feature_type| {
                feature_type.get_display_state(
                    fixture_id,
                    feature_configs,
                    channels,
                    preset_handler,
                )
            })
            .filter_map(|display_state| display_state.ok())
            .collect::<Vec<_>>())
    }
}
