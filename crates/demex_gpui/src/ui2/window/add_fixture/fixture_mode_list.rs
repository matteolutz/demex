use gpui::{Context, IntoElement, ParentElement, SharedString, Styled};
use gpui_component::{
    ActiveTheme, IndexPath, h_flex,
    label::Label,
    list::{ListDelegate, ListItem, ListState},
};
use itertools::Itertools;

pub struct FixtureModeTableEntry {
    pub name: SharedString,
}

pub struct FixtureModeListDelegate {
    pub items: Vec<FixtureModeTableEntry>,
    selected_index: Option<IndexPath>,
}

impl FixtureModeListDelegate {
    pub fn new(items: impl IntoIterator<Item = FixtureModeTableEntry>) -> Self {
        Self {
            items: items
                .into_iter()
                .sorted_by(|a, b| a.name.cmp(&b.name))
                .collect(),
            selected_index: None,
        }
    }

    pub fn update_items(
        &mut self,
        items: impl IntoIterator<Item = FixtureModeTableEntry>,
        cx: &mut Context<ListState<Self>>,
    ) {
        self.items = items
            .into_iter()
            .sorted_by(|a, b| a.name.cmp(&b.name))
            .collect();
        self.selected_index = None;
        cx.notify();
    }
}

impl ListDelegate for FixtureModeListDelegate {
    type Item = ListItem;

    fn items_count(&self, _section: usize, _cx: &gpui::App) -> usize {
        self.items.len()
    }

    fn render_empty(
        &mut self,
        _window: &mut gpui::Window,
        cx: &mut Context<ListState<Self>>,
    ) -> impl gpui::IntoElement {
        h_flex()
            .size_full()
            .justify_center()
            .text_color(cx.theme().muted_foreground.opacity(0.6))
            .child("Please select a fixture type")
            .into_any_element()
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
