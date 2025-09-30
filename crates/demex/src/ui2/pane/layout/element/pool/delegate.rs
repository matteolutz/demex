use std::ops::Range;

use gpui::App;

pub struct PoolItemData {
    pub name: String,
}

pub trait PoolDelegate: Sized + 'static {
    type PoolItemId;
    type PoolItem;

    fn get_title(&self) -> String;
    fn get_ids(&self, cx: &mut App, range: Range<usize>) -> Vec<Self::PoolItemId>;
    fn get_item_data(&self, id: &Self::PoolItemId, cx: &mut App) -> Option<PoolItemData>;
}
