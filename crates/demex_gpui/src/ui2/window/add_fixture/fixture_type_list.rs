use demex_core::uuid::Uuid;
use gpui::{ParentElement, SharedString};
use gpui_component::{
    IndexPath,
    label::Label,
    list::{ListDelegate, ListItem},
};
use itertools::Itertools;

pub struct FixtureTypeTableEntry {
    pub name: SharedString,
    pub fixture_type_id: Uuid,
}

pub struct FixtureTypeListDelegate {
    pub items: Vec<FixtureTypeTableEntry>,
    selected_index: Option<IndexPath>,
}

impl FixtureTypeListDelegate {
    pub fn new(items: impl IntoIterator<Item = FixtureTypeTableEntry>) -> Self {
        Self {
            items: items
                .into_iter()
                .sorted_by(|a, b| a.name.cmp(&b.name))
                .collect(),
            selected_index: None,
        }
    }
}

impl ListDelegate for FixtureTypeListDelegate {
    type Item = ListItem;

    fn items_count(&self, _section: usize, _cx: &gpui::App) -> usize {
        self.items.len()
    }

    fn render_item(
        &mut self,
        ix: IndexPath,
        _window: &mut gpui::Window,
        _cx: &mut gpui::Context<gpui_component::list::ListState<Self>>,
    ) -> Option<Self::Item> {
        self.items.get(ix.row).map(|item| {
            ListItem::new(ix)
                .child(Label::new(item.name.clone()))
                .selected(Some(ix) == self.selected_index)
        })
    }

    fn set_selected_index(
        &mut self,
        ix: Option<IndexPath>,
        _window: &mut gpui::Window,
        cx: &mut gpui::Context<gpui_component::list::ListState<Self>>,
    ) {
        self.selected_index = ix;
        cx.notify();
    }
}
