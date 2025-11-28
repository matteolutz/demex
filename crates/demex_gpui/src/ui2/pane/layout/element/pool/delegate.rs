use std::ops::Range;

use gpui::App;

pub struct PoolItemData {
    pub name: String,
}

pub trait PoolDelegate: Sized + 'static {
    type PoolItemId: Copy + Clone;
    type PoolItem;
    /// Default action will be called, when the user does a single left click on the item.
    /// All other actions are handled by the QuickActionMenu.
    type PoolItemAction: Copy + Clone + ToString + Default;

    fn get_title(&self) -> String;
    fn get_ids(&self, cx: &mut App, range: Range<usize>) -> Vec<Self::PoolItemId>;
    fn get_item_data(&self, id: Self::PoolItemId, cx: &mut App) -> Option<PoolItemData>;
    fn on_action(&self, id: Self::PoolItemId, action: Self::PoolItemAction, cx: &mut App);
}
