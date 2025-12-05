use serde::{Deserialize, Serialize};

use crate::channel3::feature::feature_type::FixtureChannel3FeatureType;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Default, strum::EnumString)]
pub enum DemexExecutorFaderFunction {
    #[default]
    Intensity,
    Speed,
    FadeAll,
    FadeFeatures(Vec<FixtureChannel3FeatureType>),
}
