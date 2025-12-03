use gpui::{AnyElement, IntoElement, ParentElement, Render, SharedString, Styled, div};
use gpui_component::{
    IconName, TitleBar,
    button::{Button, ButtonVariants},
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
                    "demex".into_any_element(),
                    Button::new("save")
                        .ghost()
                        .icon(IconName::Copy)
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
