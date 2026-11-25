use gpui::{App, Context, InteractiveElement, ParentElement, Render, Styled, div};
use gpui_component::setting::{SettingField, SettingGroup, SettingItem, SettingPage, Settings};

use crate::ui2::wm::edit_window::EditWindowDelegate;

mod actions {
    use gpui::{App, KeyBinding};

    pub const CONTEXT: &str = "demex-settings-window";

    gpui::actions!([QuitSettings]);

    pub fn init(cx: &mut App) {
        cx.bind_keys([
            KeyBinding::new("escape", QuitSettings, Some(CONTEXT)),
            KeyBinding::new("ctrl-m", QuitSettings, Some(CONTEXT)),
        ]);
    }
}

pub(super) fn init(cx: &mut App) {
    actions::init(cx);
}

pub struct SettingsWindow {}

impl SettingsWindow {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {}
    }
}

impl EditWindowDelegate for SettingsWindow {
    fn window_title(
        &self,
        _window: &mut gpui::Window,
        _cx: &gpui::App,
    ) -> impl Into<gpui::SharedString> {
        "Settings"
    }

    fn should_reactivate() -> bool {
        true
    }

    fn handle_save(&self, _window: &mut gpui::Window, _cx: &mut gpui::App) {
        todo!()
    }

    fn handle_discard(&self, _window: &mut gpui::Window, _cx: &mut gpui::App) {
        todo!()
    }
}

impl Render for SettingsWindow {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        div()
            .key_context(actions::CONTEXT)
            .on_action(cx.listener(|this, _: &actions::QuitSettings, _, cx| {
                this.close(cx);
            }))
            .size_full()
            .child(Settings::new("demex-settings").pages(vec![
                SettingPage::new("General").default_open(true).group(
                    SettingGroup::new().title("Test").item(SettingItem::new(
                        "Enable Test Feature",
                        SettingField::switch(|_| false, |_, _| {}),
                    )),
                ),
                SettingPage::new("Outputs").group(SettingGroup::new().title("Test")),
            ]))
    }
}
