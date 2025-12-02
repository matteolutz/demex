use gpui::{App, AppContext, Global, WindowHandle};
use gpui_component::{Root, notification::Notification};

use crate::ui2::wm::{
    app::WindowManagerAppExt,
    dock_window::{DockWindow, DockWindowConfig},
};

pub mod app;
pub mod dock_window;

pub struct WindowManager {
    /// The "main" windows of the application
    dock_windows: Vec<WindowHandle<Root>>,

    auto_quit: bool,
}

impl WindowManager {
    pub fn new(cx: &mut App) -> Self {
        cx.on_window_closed(|cx| {
            if cx.windows().is_empty() && cx.wm().auto_quit {
                cx.quit();
            }
        })
        .detach();

        Self {
            dock_windows: Vec::new(),
            auto_quit: false,
        }
    }

    /// When set to true, the application is quit when all windows are closed
    pub fn auto_quit(mut self, auto_quit: bool) -> Self {
        self.auto_quit = auto_quit;
        self
    }

    pub fn add_dock_window(&mut self, config: DockWindowConfig, cx: &mut App) {
        let window_handle = cx
            .open_window(config.gpui_window_options(), |window, cx| {
                cx.new(|cx| Root::new(cx.new(|cx| DockWindow::new(config, window, cx)), window, cx))
            })
            .expect("Dock window should be opened");

        self.dock_windows.push(window_handle);
    }

    pub fn add_dock_windows(
        &mut self,
        configs: impl IntoIterator<Item = DockWindowConfig>,
        cx: &mut App,
    ) {
        configs
            .into_iter()
            .for_each(|config| self.add_dock_window(config, cx));
    }

    fn read_dock_window<'a>(&self, handle: &'a WindowHandle<Root>, cx: &'a App) -> &'a DockWindow {
        handle
            .read(cx)
            .expect("Should read window Root")
            .view()
            .clone()
            .downcast::<DockWindow>()
            .expect("Root view should be a DockWindow")
            .read(cx)
    }

    pub fn main_dock_window<'a>(&'a self, cx: &'a App) -> (&'a WindowHandle<Root>, &'a DockWindow) {
        let handle = &self.dock_windows[0];
        (handle, self.read_dock_window(handle, cx))
    }

    pub fn main_dock_window_handle_mut(&mut self) -> &mut WindowHandle<Root> {
        &mut self.dock_windows[0]
    }

    pub fn dock_windows<'a>(
        &'a self,
        cx: &'a App,
    ) -> impl Iterator<Item = (&'a WindowHandle<Root>, &'a DockWindow)> {
        self.dock_windows
            .iter()
            .map(|handle| (handle, self.read_dock_window(handle, cx)))
    }
}

impl WindowManager {
    pub fn push_notifcation(&mut self, notification: impl Into<Notification>, cx: &mut App) {
        let wh = self.main_dock_window_handle_mut().clone();
        let notification = notification.into();

        cx.defer(move |cx| {
            wh.update(cx, |root, window, cx| {
                root.notification
                    .update(cx, |nl, cx| nl.push(notification, window, cx))
            })
            .inspect_err(|err| log::error!("{}", err))
            .expect("Should push notification");
        });
    }
}

impl Global for WindowManager {}
