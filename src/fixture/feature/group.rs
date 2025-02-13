use std::collections::HashMap;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use egui_probe::EguiProbe;
use serde::{Deserialize, Serialize};

use crate::fixture::channel2::channel_type::FixtureChannelType;

#[derive(Debug, Copy, Clone, PartialEq, Eq, EnumIter)]
pub enum DefaultFeatureGroup {
    Intensity,
    Position,
    Color,
    Beam,
    Focus,
    Control,
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
        }
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
        DefaultFeatureGroup::iter()
            .map(|default_feature_group| {
                (
                    default_feature_group.id(),
                    FeatureGroup {
                        id: default_feature_group.id(),
                        name: format!("{:?}", default_feature_group),
                        channel_types: FixtureChannelType::iter()
                            .filter(|channel_type| {
                                channel_type.get_default_feature_group().is_some_and(
                                    |feature_group| feature_group == default_feature_group,
                                )
                            })
                            .collect::<Vec<_>>(),
                    },
                )
            })
            .collect::<HashMap<_, _>>()
    }
}
