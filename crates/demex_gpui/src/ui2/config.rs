use gpui::{App, BorrowAppContext, Global};

#[derive(Default)]
pub struct DemexUiConfig {
    pub touchscreen_mode: bool,
}

impl DemexUiConfig {
    pub fn ui_size(&self) -> gpui_component::Size {
        if self.touchscreen_mode {
            gpui_component::Size::Large
        } else {
            Default::default()
        }
    }
}

impl Global for DemexUiConfig {}

pub trait AppConfigExt {
    fn ui_config(&self) -> &DemexUiConfig;
    fn update_ui_config<R, F: FnOnce(&mut DemexUiConfig, &mut App) -> R>(&mut self, f: F) -> R;
}

impl AppConfigExt for App {
    fn ui_config(&self) -> &DemexUiConfig {
        self.global()
    }

    fn update_ui_config<R, F: FnOnce(&mut DemexUiConfig, &mut App) -> R>(&mut self, f: F) -> R {
        self.update_global(f)
    }
}
