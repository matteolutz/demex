use std::ops::Range;

use demex_core::{
    channel3::feature::feature_group::FixtureChannel3FeatureGroup,
    command::parser::nodes::{
        action::{
            Action, ValueOrRange,
            functions::set_function::{SelectionOrSelector, SetFixturePresetArgs},
        },
        object::Object,
    },
    event::DemexEvent,
    presets::preset::{FixturePreset, FixturePresetId},
};
use gpui::{App, Context, Window};

use crate::{
    engine::DemexEngineHandler,
    ui2::pane::layout::element::pool::{
        Pool,
        delegate::{PoolDelegate, PoolItemData},
    },
};

pub struct PresetPool {
    feature_group: FixtureChannel3FeatureGroup,
}

impl PresetPool {
    pub fn new(
        feature_group: FixtureChannel3FeatureGroup,
        _window: &mut Window,
        cx: &mut Context<Pool<PresetPool>>,
    ) -> Self {
        let event_handler = DemexEngineHandler::event_handler(cx);

        cx.subscribe(&event_handler, |_, _, event, cx| match event {
            DemexEvent::ObjectPropertyChanged(object, _) if matches!(object, Object::Preset(_)) => {
                cx.notify();
            }
            _ => {}
        })
        .detach();

        Self { feature_group }
    }
}

#[derive(Copy, Clone, strum_macros::Display, Default)]
pub enum PresetPoolAction {
    #[default]
    Apply,
}

impl PoolDelegate for PresetPool {
    type PoolItemId = FixturePresetId;
    type PoolItem = FixturePreset;
    type PoolItemAction = PresetPoolAction;

    fn get_title(&self) -> String {
        self.feature_group.to_string()
    }

    fn get_ids(&self, _cx: &mut App, range: Range<usize>) -> Vec<Self::PoolItemId> {
        range
            .map(|idx| FixturePresetId {
                feature_group: self.feature_group,
                preset_id: idx as u32,
            })
            .collect::<Vec<_>>()
    }

    fn get_item_data(
        &self,
        id: Self::PoolItemId,
        cx: &mut App,
    ) -> Option<super::delegate::PoolItemData> {
        DemexEngineHandler::engine(cx).preset_handler().read(|ph| {
            ph.get_preset(id).ok().map(|p| PoolItemData {
                name: p.name().to_string(),
            })
        })
    }

    fn on_action(&self, id: Self::PoolItemId, action: Self::PoolItemAction, cx: &mut App) {
        match action {
            PresetPoolAction::Apply => {
                DemexEngineHandler::engine(cx).exec_ui(Action::SetFixturePreset(
                    SetFixturePresetArgs {
                        selection_or_selector: SelectionOrSelector::Current,
                        preset_id: ValueOrRange::Single(id),
                    },
                ));
            }
        }
    }
}
