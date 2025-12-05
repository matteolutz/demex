use gpui::{App, AppContext, Context, Entity, InteractiveElement, ParentElement, Render, Styled};
use gpui_component::{
    setting::{SettingField, SettingGroup, SettingItem, SettingPage, Settings},
    v_flex,
};

use crate::ui2::{
    titlebar::{DemexTitleBar, DemexTitleBarConfig},
    wm::edit_window::EditWindowDelegate,
};

mod actions {
    use gpui::{App, KeyBinding};

    pub const CONTEXT: &str = "demex-settings-window";

    gpui::actions!([Escape]);

    pub fn init(cx: &mut App) {
        cx.bind_keys([
            KeyBinding::new("escape", Escape, Some(CONTEXT)),
            KeyBinding::new("ctrl-m", Escape, Some(CONTEXT)),
        ]);
    }
}

pub(super) fn init(cx: &mut App) {
    actions::init(cx);
}

pub struct SettingsWindow {
    titlebar: Entity<DemexTitleBar>,
}

impl SettingsWindow {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            titlebar: cx.new(|_| {
                DemexTitleBar::new(DemexTitleBarConfig::SettingsWindow("Settings".into()))
            }),
        }
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
        v_flex()
            .key_context(actions::CONTEXT)
            .on_action(cx.listener(|this, _: &actions::Escape, _, cx| {
                this.close(cx);
            }))
            .size_full()
            .child(self.titlebar.clone())
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
