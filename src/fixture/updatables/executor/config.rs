use egui_probe::EguiProbe;
use serde::{Deserialize, Serialize};

use crate::fixture::{
    effect::feature::runtime::FeatureEffectRuntime, selection::FixtureSelection,
    sequence::runtime::SequenceRuntime,
};

#[derive(Debug, Serialize, Deserialize, EguiProbe, Clone)]
pub enum ExecutorConfig {
    Sequence {
        runtime: SequenceRuntime,
    },
    FeatureEffect {
        runtime: FeatureEffectRuntime,
        selection: FixtureSelection,
    },
}
