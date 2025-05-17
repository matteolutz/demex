use std::str::FromStr;

use strum::IntoEnumIterator;

use super::feature_type::FixtureChannel3FeatureType;

#[derive(
    Debug, Copy, Clone, PartialEq, Eq, Hash, Default, PartialOrd, Ord, strum_macros::EnumIter,
)]
#[cfg_attr(feature = "ui", derive(egui_probe::EguiProbe))]
pub enum FixtureChannel3FeatureGroup {
    #[default]
    Dimmer,

    Position,
    Gobo,
    Color,
    Beam,
    Focus,
    Control,
    Shapers,
    Video,
}

impl FixtureChannel3FeatureGroup {
    pub fn features(&self) -> impl Iterator<Item = FixtureChannel3FeatureType> + '_ {
        FixtureChannel3FeatureType::iter().filter(|feature| feature.feature_group() == *self)
    }
}

impl FromStr for FixtureChannel3FeatureGroup {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Dimmer" => Ok(Self::Dimmer),
            "Position" => Ok(Self::Position),
            "Gobo" => Ok(Self::Gobo),
            "Color" => Ok(Self::Color),
            "Beam" => Ok(Self::Beam),
            "Focus" => Ok(Self::Focus),
            "Control" => Ok(Self::Control),
            "Shapers" => Ok(Self::Shapers),
            "Video" => Ok(Self::Video),
            _ => Err(()),
        }
    }
}

impl TryFrom<u32> for FixtureChannel3FeatureGroup {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Dimmer),
            1 => Ok(Self::Position),
            2 => Ok(Self::Gobo),
            3 => Ok(Self::Color),
            4 => Ok(Self::Beam),
            5 => Ok(Self::Focus),
            6 => Ok(Self::Control),
            7 => Ok(Self::Shapers),
            8 => Ok(Self::Video),
            _ => Err(()),
        }
    }
}

impl From<FixtureChannel3FeatureGroup> for u32 {
    fn from(value: FixtureChannel3FeatureGroup) -> Self {
        match value {
            FixtureChannel3FeatureGroup::Dimmer => 0,
            FixtureChannel3FeatureGroup::Position => 1,
            FixtureChannel3FeatureGroup::Gobo => 2,
            FixtureChannel3FeatureGroup::Color => 3,
            FixtureChannel3FeatureGroup::Beam => 4,
            FixtureChannel3FeatureGroup::Focus => 5,
            FixtureChannel3FeatureGroup::Control => 6,
            FixtureChannel3FeatureGroup::Shapers => 7,
            FixtureChannel3FeatureGroup::Video => 8,
        }
    }
}

impl FixtureChannel3FeatureGroup {
    pub fn name(&self) -> &str {
        match self {
            Self::Dimmer => "Dimmer",
            Self::Position => "Position",
            Self::Gobo => "Gobo",
            Self::Color => "Color",
            Self::Beam => "Beam",
            Self::Focus => "Focus",
            Self::Control => "Control",
            Self::Shapers => "Shapers",
            Self::Video => "Video",
        }
    }

    pub fn default_feature(&self) -> FixtureChannel3FeatureType {
        match self {
            Self::Dimmer => FixtureChannel3FeatureType::Dimmer,
            Self::Position => FixtureChannel3FeatureType::PanTilt,
            Self::Gobo => FixtureChannel3FeatureType::Gobo,
            Self::Color => FixtureChannel3FeatureType::Rgb,
            Self::Beam => FixtureChannel3FeatureType::Beam,
            Self::Focus => FixtureChannel3FeatureType::Focus,
            Self::Control => FixtureChannel3FeatureType::Control,
            Self::Shapers => FixtureChannel3FeatureType::Shapers,
            Self::Video => FixtureChannel3FeatureType::Video,
        }
    }
}

impl std::fmt::Display for FixtureChannel3FeatureGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}
