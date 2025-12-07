use std::sync::Arc;

use demex_core::{
    channel3::feature::feature_group::FixtureChannel3FeatureGroup, utils::version::VERSION_STR,
};
use gpui::{
    App, AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Subscription,
    Window, WindowOptions,
};
use gpui_component::{
    ActiveTheme, Root, TitleBar,
    dock::{DockArea, DockAreaState, DockItem, DockPlacement},
    h_flex, v_flex,
};
use serde::{Deserialize, Serialize};

use crate::{
    engine::showfile::DemexShowFileManager,
    ui2::{
        ext::GpuiContextExtension,
        panels::{
            command::CommandPanel,
            fixture_list::FixtureListPanel,
            fixture_selection::FixtureSelectionPanel,
            layout_view::LayoutViewPanel,
            performance::PerformancePanel,
            pool::{PoolPanel, pool_type::PoolType},
        },
        titlebar::DemexTitleBar,
        wm::DEMEX_APP_ID,
    },
};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DockWindowConfig {
    pub dock_area_state: DockAreaState,
}

impl DockWindowConfig {
    pub(super) fn gpui_window_options() -> WindowOptions {
        WindowOptions {
            titlebar: Some(TitleBar::title_bar_options()),
            app_id: Some(DEMEX_APP_ID.to_string()),
            ..Default::default()
        }
    }
}

pub struct DockWindow {
    title_bar: Entity<DemexTitleBar>,
    dock_area: Entity<DockArea>,

    _subscriptions: Vec<Subscription>,
}

impl DockWindow {
    fn apply_default_dock_area(da: &mut DockArea, window: &mut Window, cx: &mut Context<DockArea>) {
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
        da.add_panel(
            Arc::new(cx.new(|cx| {
                PoolPanel::new(PoolType::Preset(FixtureChannel3FeatureGroup::Dimmer), cx)
            })),
            DockPlacement::Center,
            None,
            window,
            cx,
        );
        da.add_panel(
            Arc::new(cx.new(|cx| {
                PoolPanel::new(PoolType::Preset(FixtureChannel3FeatureGroup::Color), cx)
            })),
            DockPlacement::Center,
            None,
            window,
            cx,
        );
        da.add_panel(
            Arc::new(cx.new(|cx| {
                PoolPanel::new(PoolType::Preset(FixtureChannel3FeatureGroup::Position), cx)
            })),
            DockPlacement::Center,
            None,
            window,
            cx,
        );
        da.add_panel(
            Arc::new(cx.new(|cx| PoolPanel::new(PoolType::Sequence, cx))),
            DockPlacement::Center,
            None,
            window,
            cx,
        );
        da.add_panel(
            Arc::new(cx.new(|cx| PoolPanel::new(PoolType::Executor, cx))),
            DockPlacement::Center,
            None,
            window,
            cx,
        );
        da.add_panel(
            Arc::new(cx.new(|cx| LayoutViewPanel::new(window, cx))),
            DockPlacement::Center,
            None,
            window,
            cx,
        );

        let command_panel = cx.new(|cx| CommandPanel::new(window, cx));
        da.set_bottom_dock(
            DockItem::panel(Arc::new(command_panel)),
            Some(130.0.into()),
            true,
            window,
            cx,
        );

        let performance_panel = cx.new(|cx| PerformancePanel::new(window, cx));
        da.set_right_dock(
            DockItem::panel(Arc::new(performance_panel)),
            Some(300.0.into()),
            true,
            window,
            cx,
        );
    }
}

impl DockWindow {
    pub fn new(
        config: Option<DockWindowConfig>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let dock_area = cx.new(|cx| {
            let mut da = DockArea::new("dock-area", Some(5), window, cx);

            let should_load_default = config
                .map(|config| da.load(config.dock_area_state, window, cx).ok())
                .is_none();

            if should_load_default {
                Self::apply_default_dock_area(&mut da, window, cx);
            }

            da
        });

        let _subscriptions = vec![cx.observe_and_notify(&DemexShowFileManager::last_autosave(cx))];

        Self {
            title_bar: cx.new(|cx| DemexTitleBar::dock_window(cx)),
            dock_area,
            _subscriptions,
        }
    }

    pub fn update_config(&self, state: DockAreaState, window: &mut Window, cx: &mut App) {
        let _ = self
            .dock_area
            .update(cx, |dock_area, cx| dock_area.load(state, window, cx));
    }

    pub fn reset_config(&self, window: &mut Window, cx: &mut App) {
        let _: gpui::Result<()> = self.dock_area.update(cx, |dock_area, cx| {
            dock_area.load(DockAreaState::default(), window, cx)?;
            Self::apply_default_dock_area(dock_area, window, cx);
            Ok(())
        });
    }

    pub fn dump_config(&self, cx: &App) -> DockWindowConfig {
        DockWindowConfig {
            dock_area_state: self.dock_area.read(cx).dump(cx),
        }
    }

    fn _focus_panel(_dock_item: &mut DockItem, _panel_name: &str) -> bool {
        todo!("focus panel")
    }

    pub fn focus_panel(&mut self, panel_name: &str, window: &mut Window, cx: &mut App) -> bool {
        self.dock_area.update(cx, |da, cx| {
            let mut center_item = da.items().clone();

            if Self::_focus_panel(&mut center_item, panel_name) {
                da.set_center(center_item, window, cx);
                true
            } else {
                false
            }
        })
    }
}

impl DockWindow {
    pub fn render_status_bar(
        &self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let last_autosave = DemexShowFileManager::last_autosave(cx).read(cx).clone();

        h_flex()
            .justify_between()
            .items_center()
            .px_4()
            .w_full()
            .h_6()
            .border_t_1()
            .border_color(cx.theme().border)
            .text_color(cx.theme().muted_foreground)
            .text_sm()
            .child(format!("demex v{} ({})", VERSION_STR, env!("GIT_HASH")))
            .child(format!(
                "Last autosave: {}",
                last_autosave
                    .map(|la| format!("{} seconds ago", la.elapsed().as_secs()))
                    .unwrap_or_else(|| "-".to_string())
            ))
    }
}

impl Render for DockWindow {
    fn render(
        &mut self,
        window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        v_flex()
            .size_full()
            .child(self.title_bar.clone())
            .child(self.dock_area.clone())
            .child(self.render_status_bar(window, cx))
            .children(Root::render_dialog_layer(window, cx))
            .children(Root::render_sheet_layer(window, cx))
            .children(Root::render_notification_layer(window, cx))
    }
}
