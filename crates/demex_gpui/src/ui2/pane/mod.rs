use std::sync::Arc;

use gpui::prelude::*;
use gpui::{Context, Entity, Render};
use gpui_component::dock::{DockArea, DockItem, DockPlacement};
use gpui_component::{Root, v_flex};

use crate::ui2::pane::panels::command::CommandPanel;
use crate::ui2::pane::panels::fixture_list::FixtureListPanel;
use crate::ui2::pane::panels::fixture_selection::FixtureSelectionPanel;
use crate::ui2::pane::panels::performance::PerformancePanel;
use crate::ui2::titlebar::DemexTitleBar;

mod panels;

pub struct MainPane {
    title_bar: Entity<DemexTitleBar>,
    dock_area: Entity<DockArea>,
}

impl MainPane {
    pub fn new(window: &mut gpui::Window, cx: &mut Context<Self>) -> Self {
        let title_bar = cx.new(|_| Default::default());

        let dock_area = cx.new(|cx| {
            let mut da = DockArea::new("demex-main-dock", Some(5), window, cx);

            da.add_panel(
                Arc::new(cx.new(|cx| FixtureSelectionPanel::new(cx))),
                DockPlacement::Center,
                None,
                window,
                cx,
            );
            da.add_panel(
                Arc::new(cx.new(|cx| FixtureListPanel::new(window, cx))),
                DockPlacement::Center,
                None,
                window,
                cx,
            );

            let command_panel = cx.new(|cx| CommandPanel::new(window, cx));
            da.set_bottom_dock(
                DockItem::panel(Arc::new(command_panel)),
                Some(100.0.into()),
                true,
                window,
                cx,
            );

            let performance_panel = cx.new(|cx| PerformancePanel::new(window, cx));
            da.set_right_dock(
                DockItem::panel(Arc::new(performance_panel)),
                Some(200.0.into()),
                true,
                window,
                cx,
            );

            da
        });

        Self {
            title_bar,
            dock_area,
        }
    }
}

impl Render for MainPane {
    fn render(
        &mut self,
        window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        v_flex()
            .size_full()
            .child(self.title_bar.clone())
            .child(self.dock_area.clone())
            .children(Root::render_dialog_layer(window, cx))
            .children(Root::render_sheet_layer(window, cx))
            .children(Root::render_notification_layer(window, cx))
    }
}
