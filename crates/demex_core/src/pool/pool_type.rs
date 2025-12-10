use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

use crate::channel3::feature::feature_group::FixtureChannel3FeatureGroup;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PoolType {
    Executor,
    Preset(FixtureChannel3FeatureGroup),
    Sequence,
    Group,
    Macro,
}

impl Default for PoolType {
    fn default() -> Self {
        PoolType::Preset(FixtureChannel3FeatureGroup::default())
    }
}

impl PoolType {
    pub fn all() -> Vec<PoolType> {
        [Self::Executor, Self::Sequence, Self::Group, Self::Macro]
            .into_iter()
            .chain(FixtureChannel3FeatureGroup::iter().map(|fg| Self::Preset(fg)))
            .collect()
    }
}
