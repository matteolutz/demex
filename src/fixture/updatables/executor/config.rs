use serde::{Deserialize, Serialize};

use crate::fixture::{
    effect::feature::runtime::FeatureEffectRuntime, selection::FixtureSelection,
    sequence::runtime::SequenceRuntime,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ui", derive(egui_probe::EguiProbe))]
pub enum ExecutorConfig {
    Sequence {
        runtime: SequenceRuntime,
    },
    FeatureEffect {
        runtime: FeatureEffectRuntime,
        selection: FixtureSelection,
    },
}
