use gpui::{
    AnyElement, App, Context, InteractiveElement, IntoElement, ParentElement, Render, SharedString,
    Styled, Subscription, Window, div,
};
use gpui_component::{
    Sizable, TitleBar,
    button::{Button, ButtonVariants},
    h_flex,
    menu::DropdownMenu,
};

use crate::{
    app,
    engine::showfile::DemexShowFileManager,
    ui2::{
        ext::GpuiContextExtension,
        window::{outputs::OutputsConfigWindow, settings::SettingsWindow},
        wm::{WindowManager, app::WindowManagerAppExt, edit_window::WindowManagerExtension},
    },
};

mod actions {
    gpui::actions!(titlebar, [NewFile, ResetView]);
}

#[derive(Default)]
pub enum DemexTitleBarConfig {
    #[default]
    DockWindow,

    SettingsWindow(SharedString),
}

impl DemexTitleBarConfig {
    pub fn into_children(
        &self,
        _window: &mut Window,
        cx: &mut App,
    ) -> impl IntoIterator<Item = AnyElement> {
        let has_showfile_name = DemexShowFileManager::current_file_path(cx)
            .read(cx)
            .is_some();

        let showfile_name = DemexShowFileManager::current_file_path(cx)
            .read(cx)
            .as_ref()
            .and_then(|path| path.file_name())
            .and_then(|name| name.to_str().map(|s| s.to_string()))
            .unwrap_or_else(|| "Untitled show".to_string());

        match self {
            Self::DockWindow => {
                vec![
                    div().text_xl().child("demex").into_any_element(),
                    h_flex()
                        .px_4()
                        .py_2()
                        .gap_2()
                        .child(
                            Button::new("file-menu")
                                .small()
                                .link()
                                .label("File")
                                .dropdown_menu(move |menu, _, _| {
                                    menu.menu("New", Box::new(actions::NewFile))
                                        .separator()
                                        .menu("Save", Box::new(app::actions::Save))
                                        .menu("Save As", Box::new(app::actions::SaveAs))
                                        .separator()
                                        .menu_with_enable(
                                            "Open",
                                            Box::new(app::actions::Open),
                                            true,
                                        )
                                        .separator()
                                        .menu("Reload UI", Box::new(app::actions::ReloadUi))
                                        .menu_with_enable(
                                            "Reload",
                                            Box::new(app::actions::Reload),
                                            has_showfile_name,
                                        )
                                }),
                        )
                        .child(
                            Button::new("view-menu")
                                .small()
                                .link()
                                .label("View")
                                .dropdown_menu(move |menu, _, _| {
                                    menu.menu("Reset Layout", Box::new(actions::ResetView))
                                }),
                        )
                        .child(
                            Button::new("settings")
                                .small()
                                .link()
                                .label("Settings")
                                .on_click(|_, _, cx| {
                                    WindowManager::open_edit_window::<SettingsWindow>(cx, |cx| {
                                        SettingsWindow::new(cx)
                                    });
                                }),
                        )
                        .child(
                            Button::new("outputs")
                                .small()
                                .link()
                                .label("Outputs")
                                .on_click(|_, _, cx| {
                                    WindowManager::open_edit_window::<OutputsConfigWindow>(
                                        cx,
                                        |cx| OutputsConfigWindow::new(cx),
                                    );
                                }),
                        )
                        .into_any_element(),
                    h_flex()
                        .px_8()
                        .justify_center()
                        .child(showfile_name)
                        .into_any_element(),
                ]
            }
            Self::SettingsWindow(title) => {
                vec![title.clone().into_any_element()]
            }
        }
    }
}

pub struct DemexTitleBar {
    config: DemexTitleBarConfig,
    _subscriptions: Vec<Subscription>,
}

impl DemexTitleBar {
    pub fn new(config: DemexTitleBarConfig, cx: &mut Context<Self>) -> Self {
        let _subscriptions =
            vec![cx.observe_and_notify(&DemexShowFileManager::current_file_path(cx))];

        Self {
            config,
            _subscriptions,
        }
    }

    pub fn dock_window(cx: &mut Context<Self>) -> Self {
        Self::new(DemexTitleBarConfig::DockWindow, cx)
    }

    pub fn settings(title: impl Into<SharedString>, cx: &mut Context<Self>) -> Self {
        Self::new(DemexTitleBarConfig::SettingsWindow(title.into()), cx)
    }
}

impl DemexTitleBar {
    fn handle_new(_: &actions::NewFile, _: &mut Window, cx: &mut App) {
        DemexShowFileManager::load_empty_show(cx);
    }

    fn handle_reset_view(_: &actions::ResetView, _window: &mut Window, cx: &mut App) {
        cx.defer(|cx| cx.update_wm(|wm, cx| wm.reset_dock_window_configs(cx)));
    }
}

impl Render for DemexTitleBar {
    fn render(
        &mut self,
        window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        TitleBar::new().child(
            div()
                .w_full()
                .on_action(Self::handle_new)
                .on_action(Self::handle_reset_view)
                .flex()
                .justify_start()
                .items_center()
                .gap_2()
                .children(self.config.into_children(window, cx)),
        )
    }
}
