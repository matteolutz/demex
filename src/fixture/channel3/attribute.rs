use super::feature::feature_type::FixtureChannel3FeatureType;

lazy_static::lazy_static! {
    pub static ref ATTRIBUTE_WILDCARD_REGEX: regex::Regex = regex::Regex::new(r"(\d+)").unwrap();
}

#[derive(Debug, Copy, Clone, strum_macros::EnumIter)]
pub enum FixtureChannel3Attribute {
    Dimmer,

    Pan,
    Tilt,
    PanRotate,
    TiltRotate,
    PositionEffect,
    PositionEffectRate,
    PositionEffectFade,
}

impl FixtureChannel3Attribute {
    pub fn feature(&self) -> FixtureChannel3FeatureType {
        match self {
            Self::Dimmer => FixtureChannel3FeatureType::Dimmer,
            Self::Pan
            | Self::Tilt
            | Self::PanRotate
            | Self::TiltRotate
            | Self::PositionEffect
            | Self::PositionEffectRate
            | Self::PositionEffectFade => FixtureChannel3FeatureType::PanTilt,
        }
    }
}

impl FixtureChannel3Attribute {
    pub fn attribute_matches(fixture_attribute_name: &str, attribute_name: &str) -> bool {
        let fixture_attribute_name =
            ATTRIBUTE_WILDCARD_REGEX.replace(fixture_attribute_name, "(n)");
        let fixture_attribute_name =
            ATTRIBUTE_WILDCARD_REGEX.replace(&fixture_attribute_name, "(m)");

        fixture_attribute_name == attribute_name
    }
}

impl std::fmt::Display for FixtureChannel3Attribute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Dimmer => write!(f, "Dimmer"),
            Self::Pan => write!(f, "Pan"),
            Self::Tilt => write!(f, "Tilt"),
            Self::PanRotate => write!(f, "PanRotate"),
            Self::TiltRotate => write!(f, "TiltRotate"),
            Self::PositionEffect => write!(f, "PositionEffect"),
            Self::PositionEffectRate => write!(f, "PositionEffectRate"),
            Self::PositionEffectFade => write!(f, "PositionEffectFade"),
        }
    }
}
