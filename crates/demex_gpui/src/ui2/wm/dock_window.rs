use std::sync::Arc;

use demex_core::utils::version::VERSION_STR;
use gpui::{
    App, AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Subscription,
    Window, WindowOptions, div, prelude::FluentBuilder,
};
use gpui_component::{
    ActiveTheme, Root, Sizable,
    dock::{DockArea, DockAreaState, DockItem, DockPlacement, PanelStyle, TabPanel},
    h_flex,
    input::InputState,
    v_flex,
};
use serde::{Deserialize, Serialize};

use crate::{
    engine::{showfile::DemexShowFileManager, state::DemexUiState},
    settings::DemexWindowSettings,
    ui2::{
        components::context::DemexContextLayer,
        config::AppConfigExt,
        ext::GpuiContextExtension,
        panels::{
            DemexPanelContextExt, DockWindowPanelType,
            attribute_editor::AttributeEditorPanel,
            command::CommandPanel,
            fixture_list::FixtureListPanel,
            fixture_selection::FixtureSelectionPanel,
            layout_view::LayoutViewPanel,
            multipool::{MultiPoolPanel, config::MultiPoolConfig},
            performance::PerformancePanel,
            sequence_editor::SequenceEditorPanel,
        },
        titlebar::{DemexTitleBar, titlebar_options},
        wm::DEMEX_APP_ID,
    },
};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DockWindowConfig {
    pub dock_area_state: DockAreaState,
}

impl DockWindowConfig {
    pub(super) fn gpui_window_options(settings: Option<DemexWindowSettings>) -> WindowOptions {
        WindowOptions {
            window_bounds: settings
                .as_ref()
                .and_then(|settings| settings.bounds)
                .map(|bounds| bounds.into()),
            display_id: None,
            titlebar: Some(titlebar_options()),
            app_id: Some(DEMEX_APP_ID.to_string()),
            ..Default::default()
        }
    }
}

pub struct DockWindow {
    title_bar: Entity<DemexTitleBar>,
    dock_area: Entity<DockArea>,

    is_main: bool,

    context_layer: Entity<DemexContextLayer>,

    _subscriptions: Vec<Subscription>,
}

impl DockWindow {
    fn apply_default_dock_area(
        da: &mut DockArea,
        is_main: bool,
        window: &mut Window,
        cx: &mut Context<DockArea>,
    ) {
        da.add_panel(
            Arc::new(cx.new_panel(|cx| FixtureSelectionPanel::new(cx))),
            DockPlacement::Center,
            None,
            window,
            cx,
        );
        da.add_panel(
            Arc::new(cx.new_panel(|cx| FixtureListPanel::new(window, cx))),
            DockPlacement::Center,
            None,
            window,
            cx,
        );

        da.add_panel(
            Arc::new(cx.new_panel(|cx| MultiPoolPanel::new(MultiPoolConfig::example(), cx))),
            DockPlacement::Center,
            None,
            window,
            cx,
        );
        da.add_panel(
            Arc::new(cx.new_panel(|cx| LayoutViewPanel::new(window, cx))),
            DockPlacement::Center,
            None,
            window,
            cx,
        );
        da.add_panel(
            Arc::new(cx.new_panel(|cx| SequenceEditorPanel::new(window, cx))),
            DockPlacement::Center,
            None,
            window,
            cx,
        );
        da.add_panel(
            Arc::new(cx.new_panel(|cx| AttributeEditorPanel::new(window, cx))),
            DockPlacement::Center,
            None,
            window,
            cx,
        );

        if is_main {
            let command_panel = cx.new_panel(|cx| CommandPanel::new(window, cx));
            da.set_bottom_dock(
                DockItem::panel(Arc::new(command_panel)),
                Some(130.0.into()),
                true,
                window,
                cx,
            );
        }

        if cfg!(debug_assertions) {
            let performance_panel = cx.new_panel(|cx| PerformancePanel::new(window, cx));
            da.set_right_dock(
                DockItem::panel(Arc::new(performance_panel)),
                Some(300.0.into()),
                true,
                window,
                cx,
            );
        }
    }

    fn init_dockarea(
        config: Option<DockWindowConfig>,
        is_main: bool,
        window: &mut Window,
        cx: &mut Context<DockArea>,
    ) -> DockArea {
        let mut da = DockArea::new("dock-area", Some(5), window, cx)
            .panel_style(PanelStyle::TabBar)
            .with_size(cx.ui_config().ui_size());

        let should_load_default = config
            .map(|config| da.load(config.dock_area_state, window, cx).ok())
            .is_none();

        if should_load_default {
            Self::apply_default_dock_area(&mut da, is_main, window, cx);
        }

        da
    }
}

impl DockWindow {
    pub fn new(
        config: Option<DockWindowConfig>,
        is_main: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let dock_area = cx.new(|cx| Self::init_dockarea(config, is_main, window, cx));

        let _subscriptions = vec![
            cx.observe_and_notify(&DemexShowFileManager::last_autosave(cx)),
            cx.observe_and_notify(&DemexUiState::highlight(cx)),
        ];

        Self {
            title_bar: cx.new(|cx| DemexTitleBar::dock_window(cx)),
            dock_area,
            context_layer: cx.new(|_| DemexContextLayer::default()),
            is_main,
            _subscriptions,
        }
    }

    pub fn update_config(&self, config: DockWindowConfig, window: &mut Window, cx: &mut App) {
        self.dock_area.update(cx, |dock_area, cx| {
            let _ = dock_area.load(config.dock_area_state, window, cx);
            cx.notify();
        });
    }

    pub fn reset_config(&self, window: &mut Window, cx: &mut App) {
        self.dock_area.update(cx, |dock_area, cx| {
            *dock_area = Self::init_dockarea(None, self.is_main, window, cx);
            cx.notify();
        });
    }

    pub fn dump_config(&self, cx: &App) -> DockWindowConfig {
        DockWindowConfig {
            dock_area_state: self.dock_area.read(cx).dump(cx),
        }
    }

    pub fn context_layer(&self) -> Entity<DemexContextLayer> {
        self.context_layer.clone()
    }

    fn _focus_panel(_dock_item: &mut DockItem, _panel_name: &str) -> bool {
        todo!("focus panel")
    }

    pub fn focus_panel(&mut self, panel_name: &str, window: &mut Window, cx: &mut App) -> bool {
        self.dock_area.update(cx, |da, cx| {
            let mut center_item = da.center().clone();

            if Self::_focus_panel(&mut center_item, panel_name) {
                da.set_center(center_item, window, cx);
                true
            } else {
                false
            }
        })
    }

    pub fn command_input_state(&self, cx: &App) -> Option<Entity<InputState>> {
        let da = self.dock_area.read(cx);

        let command_panel = da.bottom_dock().and_then(|dock| {
            let panel_view = dock.read(cx).panel();

            let tab_panel_entity = panel_view.view().view().downcast::<TabPanel>().ok()?;
            let active_panel = tab_panel_entity.read(cx).active_panel(cx)?;

            active_panel.view().downcast::<CommandPanel>().ok()
        });

        command_panel.map(|panel| panel.read(cx).command_input_state.clone())
    }

    /// Append text to the command input, also handling whitespaces
    pub fn append_to_command(&self, text: impl ToString, window: &mut Window, cx: &mut App) {
        let Some(command_input_state) = self.command_input_state(cx) else {
            return;
        };

        command_input_state.update(cx, |state, cx| {
            let value = state.value();
            let value = value.strip_suffix(" ").unwrap_or(value.as_str());

            let new_value = if value.is_empty() {
                text.to_string()
            } else {
                format!("{} {}", value, text.to_string())
            };

            state.set_value(new_value, window, cx);
        });
    }

    pub fn add_panel(
        &self,
        panel_type: DockWindowPanelType,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.dock_area.update(cx, |da, cx| {
            da.add_panel(
                match panel_type {
                    DockWindowPanelType::AttributeEditor => {
                        Arc::new(cx.new_panel(|cx| AttributeEditorPanel::new(window, cx)))
                    }
                    DockWindowPanelType::FixtureList => {
                        Arc::new(cx.new_panel(|cx| FixtureListPanel::new(window, cx)))
                    }
                    DockWindowPanelType::FixtureSelection => {
                        Arc::new(cx.new_panel(|cx| FixtureSelectionPanel::new(cx)))
                    }
                    DockWindowPanelType::Multipool => Arc::new(
                        cx.new_panel(|cx| MultiPoolPanel::new(MultiPoolConfig::default(), cx)),
                    ),
                    DockWindowPanelType::LayoutView => {
                        Arc::new(cx.new_panel(|cx| LayoutViewPanel::new(window, cx)))
                    }
                    DockWindowPanelType::SequenceEditor => {
                        Arc::new(cx.new_panel(|cx| SequenceEditorPanel::new(window, cx)))
                    }
                    DockWindowPanelType::CommandPanel => {
                        Arc::new(cx.new_panel(|cx| CommandPanel::new(window, cx)))
                    }
                    DockWindowPanelType::PerformancePanel => {
                        Arc::new(cx.new_panel(|cx| PerformancePanel::new(window, cx)))
                    }
                },
                DockPlacement::Center,
                None,
                window,
                cx,
            );
        });
        cx.notify();
    }
}

impl DockWindow {
    pub fn render_status_bar(
        &self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let last_autosave = DemexShowFileManager::last_autosave(cx).read(cx).clone();
        let highlight = DemexUiState::highlight(cx).read(cx);

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
            .child(format!("demex v{}-{}", VERSION_STR, env!("GIT_HASH")))
            .child(
                div()
                    .flex_1()
                    .flex()
                    .gap_2()
                    .px_4()
                    .when(highlight.is_some(), |this| {
                        this.child(div().text_color(cx.theme().red).child("HIGHLIGHT"))
                    }),
            )
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
            .relative()
            .size_full()
            .child(self.title_bar.clone())
            .child(self.dock_area.clone())
            .child(self.render_status_bar(window, cx))
            .child(self.context_layer.clone())
            .children(Root::render_dialog_layer(window, cx))
            .children(Root::render_sheet_layer(window, cx))
            .children(Root::render_notification_layer(window, cx))
    }
}
