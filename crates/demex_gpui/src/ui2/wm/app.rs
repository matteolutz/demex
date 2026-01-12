use gpui::{App, AsyncApp, BorrowAppContext};

use crate::ui2::wm::WindowManager;

pub trait WindowManagerAppExt {
    fn wm(&self) -> &WindowManager;

    fn update_wm<R, F>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut WindowManager, &mut Self) -> R;
}

impl WindowManagerAppExt for App {
    fn wm(&self) -> &WindowManager {
        self.global()
    }

    fn update_wm<R, F>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut WindowManager, &mut Self) -> R,
    {
        self.update_global(f)
    }
}

pub trait WindowManagerAsyncAppExt {
    fn update_wm<R, F>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut WindowManager, &mut App) -> R;
}

impl WindowManagerAsyncAppExt for AsyncApp {
    fn update_wm<R, F>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut WindowManager, &mut App) -> R,
    {
        self.update_global(f)
    }
}
