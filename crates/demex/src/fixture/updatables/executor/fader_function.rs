use serde::{Deserialize, Serialize};

use crate::fixture::channel3::feature::feature_type::FixtureChannel3FeatureType;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "ui", derive(egui_probe::EguiProbe))]
pub enum DemexExecutorFaderFunction {
    #[default]
    Intensity,
    Speed,
    FadeAll,
    FadeFeatures(Vec<FixtureChannel3FeatureType>),
}
