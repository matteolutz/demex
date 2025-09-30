use std::ops::Range;

use demex_core::{
    channel3::feature::feature_group::FixtureChannel3FeatureGroup,
    presets::preset::{FixturePreset, FixturePresetId},
};
use gpui::App;

use crate::{
    engine::DemexEngineHandler,
    ui2::pane::layout::element::pool::delegate::{PoolDelegate, PoolItemData},
};

pub struct PresetPool {
    feature_group: FixtureChannel3FeatureGroup,
}

impl PresetPool {
    pub fn new(feature_group: FixtureChannel3FeatureGroup) -> Self {
        Self { feature_group }
    }
}

impl PoolDelegate for PresetPool {
    type PoolItemId = FixturePresetId;
    type PoolItem = FixturePreset;

    fn get_title(&self) -> String {
        self.feature_group.to_string()
    }

    fn get_ids(&self, cx: &mut App, range: Range<usize>) -> Vec<Self::PoolItemId> {
        range
            .map(|idx| FixturePresetId {
                feature_group: self.feature_group,
                preset_id: idx as u32,
            })
            .collect::<Vec<_>>()
    }

    fn get_item_data(
        &self,
        id: &Self::PoolItemId,
        cx: &mut App,
    ) -> Option<super::delegate::PoolItemData> {
        DemexEngineHandler::engine(cx).preset_handler().read(|ph| {
            ph.get_preset(*id).ok().map(|p| PoolItemData {
                name: p.name().to_string(),
            })
        })
    }
}
