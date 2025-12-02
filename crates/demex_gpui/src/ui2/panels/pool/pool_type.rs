use demex_core::channel3::feature::feature_group::FixtureChannel3FeatureGroup;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum PoolType {
    Preset(FixtureChannel3FeatureGroup),

    Sequence,
    Executor,
}

impl Default for PoolType {
    fn default() -> Self {
        Self::Preset(FixtureChannel3FeatureGroup::default())
    }
}

impl std::fmt::Display for PoolType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Preset(feature_group) => write!(f, "{} Preset", feature_group),
            Self::Sequence => write!(f, "Sequence"),
            Self::Executor => write!(f, "Executor"),
        }
    }
}
