use demex_core::utils::version::VERSION_STR;
use gpui::{
    AnyElement, App, BorrowAppContext, InteractiveElement, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div,
};
use gpui_component::{
    ActiveTheme, TitleBar,
    button::{Button, ButtonVariants},
    h_flex,
    menu::DropdownMenu,
};

use crate::{
    app,
    engine::showfile::DemexShowFileManager,
    ui2::{
        window::{outputs::OutputsConfigWindow, settings::SettingsWindow},
        wm::{WindowManager, edit_window::WindowManagerExtension},
    },
};

mod actions {
    gpui::actions!(titlebar, [NewFile]);
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
        match self {
            Self::DockWindow => {
                vec![
                    h_flex()
                        .gap_1()
                        .items_center()
                        .child(div().text_xl().child("demex"))
                        .child(
                            div()
                                .text_sm()
                                .text_color(cx.theme().muted_foreground)
                                .child(format!("v{}", VERSION_STR)),
                        )
                        .into_any_element(),
                    h_flex()
                        .px_4()
                        .py_2()
                        .gap_2()
                        .child(Button::new("file-menu").link().label("File").dropdown_menu(
                            |menu, _, _| {
                                menu.menu("New", Box::new(actions::NewFile))
                                    .separator()
                                    .menu("Save", Box::new(app::actions::Save))
                                    .menu("Save As", Box::new(app::actions::SaveAs))
                                    .separator()
                                    .menu_with_enable("Open", Box::new(app::actions::Open), false)
                            },
                        ))
                        .child(Button::new("settings").link().label("Settings").on_click(
                            |_, _, cx| {
                                WindowManager::open_edit_window::<SettingsWindow>(cx, |cx| {
                                    SettingsWindow::new(cx)
                                });
                            },
                        ))
                        .child(Button::new("outputs").link().label("Outputs").on_click(
                            |_, _, cx| {
                                WindowManager::open_edit_window::<OutputsConfigWindow>(cx, |cx| {
                                    OutputsConfigWindow::new(cx)
                                });
                            },
                        ))
                        .into_any_element(),
                ]
            }
            Self::SettingsWindow(title) => {
                vec![title.clone().into_any_element()]
            }
        }
    }
}

#[derive(Default)]
pub struct DemexTitleBar {
    config: DemexTitleBarConfig,
}

impl DemexTitleBar {
    pub fn new(config: DemexTitleBarConfig) -> Self {
        Self { config }
    }

    pub fn settings(title: impl Into<SharedString>) -> Self {
        Self::new(DemexTitleBarConfig::SettingsWindow(title.into()))
    }
}

impl DemexTitleBar {
    fn handle_new(_: &actions::NewFile, _: &mut Window, cx: &mut App) {
        let _ =
            cx.update_global(|manager: &mut DemexShowFileManager, cx| manager.load_empty_show(cx));
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
                .on_action(Self::handle_new)
                .flex()
                .items_center()
                .justify_end()
                .gap_2()
                .children(self.config.into_children(window, cx)),
        )
    }
}
