use std::ops::Range;

use demex_core::{
    command::parser::nodes::{
        action::Action,
        fixture_selector::{AtomicFixtureSelector, FixtureSelector},
        object::{HomeableObject, Object},
    },
    event::DemexEvent,
    presets::group::FixtureGroup,
};
use gpui::{App, Context, Window};

use crate::{
    engine::DemexEngineHandler,
    ui2::pane::layout::element::pool::{
        Pool,
        delegate::{PoolDelegate, PoolItemData},
    },
};

pub struct GroupPool {}

impl GroupPool {
    pub fn new(_window: &mut Window, cx: &mut Context<Pool<GroupPool>>) -> Self {
        let event_handler = DemexEngineHandler::event_handler(cx);

        cx.subscribe(&event_handler, |_, _, event, cx| match event {
            DemexEvent::ObjectPropertyChanged(object, _)
            if matches!(object, Object::HomeableObject(homeable_object)
                if matches!(homeable_object, HomeableObject::FixtureSelector(selector) if selector.try_as_group_id().is_some())) => {
                    cx.notify();
            }
            _ => {}
        })
        .detach();

        Self {}
    }
}

#[derive(Copy, Clone, strum_macros::Display, Default)]
pub enum GroupPoolAction {
    #[default]
    SetCurrent,
}

impl PoolDelegate for GroupPool {
    type PoolItemId = u32;
    type PoolItem = FixtureGroup;
    type PoolItemAction = GroupPoolAction;

    fn get_title(&self) -> String {
        "Group".to_string()
    }

    fn get_ids(&self, _cx: &mut App, range: Range<usize>) -> Vec<Self::PoolItemId> {
        range.map(|idx| idx as u32).collect::<Vec<_>>()
    }

    fn get_item_data(
        &self,
        id: Self::PoolItemId,
        cx: &mut App,
    ) -> Option<super::delegate::PoolItemData> {
        DemexEngineHandler::engine(cx).preset_handler().read(|ph| {
            ph.get_group(id).ok().map(|group| PoolItemData {
                name: group.name().to_string(),
            })
        })
    }

    fn on_action(&self, id: Self::PoolItemId, action: Self::PoolItemAction, cx: &mut App) {
        match action {
            GroupPoolAction::SetCurrent => {
                DemexEngineHandler::engine(cx).exec_ui(Action::FixtureSelector(
                    FixtureSelector::Atomic(AtomicFixtureSelector::FixtureGroup(id)),
                ));
            }
        }
    }
}
