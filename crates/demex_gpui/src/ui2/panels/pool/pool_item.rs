use demex_core::pool::PoolItemName;
use gpui::App;

use crate::{
    engine::state::DemexUiState, ui2::panels::pool::pool_button::PoolItemButtonIndicatorColor,
};

pub trait PoolItemNameExt {
    fn to_name(self, cx: &App) -> String;
}

impl PoolItemNameExt for PoolItemName {
    fn to_name(self, cx: &App) -> String {
        match self {
            Self::String(name) => name,
            Self::Reference(pool_type, ref_id) => {
                let pool = DemexUiState::try_pool(pool_type, cx);
                let Some(pool_item) =
                    pool.and_then(|pool| pool.read(cx).iter().find(|item| item.id == ref_id))
                else {
                    return "Deleted reference".to_string();
                };

                pool_item.name.clone().to_name(cx)
            }
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct PoolItemState {
    pub(crate) indicator_color: PoolItemButtonIndicatorColor,
}
