use gpui::{App, BorrowAppContext};

use crate::ui2::wm::WindowManager;

pub trait WmAppExt {
    fn wm(&self) -> &WindowManager;
    fn update_wm<R, F: FnOnce(&mut WindowManager, &mut App) -> R>(&mut self, f: F) -> R;
}

impl WmAppExt for App {
    fn wm(&self) -> &WindowManager {
        self.global()
    }

    fn update_wm<R, F: FnOnce(&mut WindowManager, &mut App) -> R>(&mut self, f: F) -> R {
        self.update_global(f)
    }
}
