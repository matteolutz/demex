use gpui::{Entity, IntoElement, Point};

use crate::ui2::pane::layout::element::{
    fixture_sheet::FixtureSheet,
    performance::Performance,
    playback::Playback,
    pool::{Pool, preset::PresetPool},
};

pub mod fixture_sheet;
pub mod performance;
pub mod playback;
pub mod pool;

pub enum LayoutViewElementType {
    FixtureSheet(Entity<FixtureSheet>),
    Playback(Entity<Playback>),

    PresetPool(Entity<Pool<PresetPool>>),

    Performance(Entity<Performance>),
}

pub struct LayoutViewElement {
    pub(crate) from: Point<u16>,
    pub(crate) to: Point<u16>,
    pub(crate) element_type: LayoutViewElementType,
}

impl LayoutViewElementType {
    pub fn render(&self) -> impl IntoElement {
        match self {
            Self::FixtureSheet(fixture_sheet) => fixture_sheet.clone().into_any_element(),
            Self::Playback(playback) => playback.clone().into_any_element(),
            Self::PresetPool(preset_pool) => preset_pool.clone().into_any_element(),
            Self::Performance(performance) => performance.clone().into_any_element(),
        }
    }
}
