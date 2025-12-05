use gpui::{AnyElement, IntoElement, ParentElement, Render, SharedString, Styled, div};
use gpui_component::{
    TitleBar,
    button::{Button, ButtonVariants},
};

use crate::ui2::{
    window::outputs::OutputsConfigWindow,
    wm::{WindowManager, edit_window::WindowManagerExtension},
};

#[derive(Default)]
pub enum DemexTitleBarConfig {
    #[default]
    DockWindow,

    SettingsWindow(SharedString),
}

impl DemexTitleBarConfig {
    pub fn into_children(&self) -> impl IntoIterator<Item = AnyElement> {
        match self {
            Self::DockWindow => {
                vec![
                    div().text_lg().child("demex").into_any_element(),
                    Button::new("save").link().label("Save").into_any_element(),
                    Button::new("settings")
                        .link()
                        .label("Settings")
                        .into_any_element(),
                    Button::new("outputs")
                        .link()
                        .label("Outputs")
                        .on_click(|_, _, cx| {
                            WindowManager::open_edit_window::<OutputsConfigWindow>(cx, |cx| {
                                OutputsConfigWindow::new(cx)
                            });
                        })
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

impl Render for DemexTitleBar {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        _cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        TitleBar::new().child(
            div()
                .flex()
                .items_center()
                .justify_end()
                .gap_2()
                .children(self.config.into_children()),
        )
    }
}
