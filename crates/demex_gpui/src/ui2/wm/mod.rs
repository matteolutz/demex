use std::{any::TypeId, collections::HashMap};

use gpui::{App, AppContext, Context, Entity, Global, PromptButton, PromptLevel, WindowHandle};
use gpui_component::{Root, notification::Notification};

use crate::ui2::wm::{
    app::WindowManagerAppExt,
    dock_window::{DockWindow, DockWindowConfig},
    window::{WindowDelegate, WindowWrapper},
};

const DEMEX_APP_ID: &str = "demex";

pub mod app;
pub mod dock_window;
pub mod edit_window;
pub mod window;

#[derive(Debug, Copy, Clone)]
pub struct SingletonWindow {
    pub handle: WindowHandle<Root>,
    pub is_edited: bool,
}

impl SingletonWindow {
    pub fn new(handle: WindowHandle<Root>) -> Self {
        Self {
            handle,
            is_edited: false,
        }
    }
}

pub struct WindowManager {
    /// The "main" windows of the application
    dock_windows: Vec<WindowHandle<Root>>,

    /// Collection of all "singleton" windows
    /// (i.e. settings, ...)
    singleton_windows: HashMap<TypeId, SingletonWindow>,

    auto_quit: bool,
}

impl WindowManager {
    pub fn new(cx: &mut App) -> Self {
        cx.on_window_closed(|cx| {
            let n_dock_windows = cx.update_wm(|wm, cx| {
                wm.dock_windows
                    .iter()
                    // why ever this thing needs &mut App??
                    .filter(|h| h.is_active(cx).is_some())
                    .count()
            });

            if n_dock_windows == 0 && cx.wm().auto_quit {
                cx.quit();
            }
        })
        .detach();

        Self {
            dock_windows: Vec::new(),
            singleton_windows: HashMap::new(),
            auto_quit: false,
        }
    }

    /// When set to true, the application is quit when all dock ("main") windows are closed
    pub fn auto_quit(mut self, auto_quit: bool) -> Self {
        self.auto_quit = auto_quit;
        self
    }

    // Singleton windows
    pub fn open_singleton_window<D: WindowDelegate>(
        &mut self,
        cx: &mut App,
        data: impl FnOnce(&mut Context<D::InitData>) -> D::InitData,
    ) {
        let type_id = TypeId::of::<D>();

        if self.singleton_windows.contains_key(&type_id) {
            return;
        }

        let handle = WindowWrapper::open(cx, |window, cx| {
            window.on_window_should_close(cx, move |_, cx| {
                cx.update_wm(|wm, cx| wm.request_close_singleton_window::<D>(cx))
            });

            D::create(window, cx, data)
        });

        self.singleton_windows
            .insert(type_id, SingletonWindow::new(handle));
    }

    pub fn set_singleton_window_edited<D: WindowDelegate>(&mut self, cx: &mut App, edited: bool) {
        let type_id = TypeId::of::<D>();
        self.singleton_windows.entry(type_id).and_modify(|sw| {
            let _ = cx.update_window(sw.handle.into(), |_, window, _| {
                window.set_window_edited(edited);
            });
            sw.is_edited = edited;
        });
    }

    pub fn request_close_singleton_window<D: WindowDelegate>(&mut self, cx: &mut App) -> bool {
        let type_id = TypeId::of::<D>();

        let Some(&singleton_window) = self.singleton_windows.get(&type_id) else {
            // we don't know the window, so close it
            return true;
        };

        let close_window = move |cx: &mut App| {
            cx.defer(move |cx| {
                singleton_window
                    .handle
                    .clone()
                    .update(cx, |_, window, _| window.remove_window())
                    .expect("should update window");
                cx.update_wm(|wm, _| wm.singleton_windows.remove(&type_id));
            });
        };

        if !singleton_window.is_edited {
            close_window(cx);
            return true;
        }

        if singleton_window.is_edited {
            cx.defer(move |cx| {
                let answer = singleton_window
                    .handle
                    .update(cx, |root, window, cx| {
                        let delegate = root
                            .view()
                            .clone()
                            .downcast::<WindowWrapper<D>>()
                            .expect("Root view should be a WindowWrapper");

                        let window_title =
                            delegate.read_with(cx, |d, cx| d.window_title(window, cx).into());

                        window.prompt(
                            PromptLevel::Warning,
                            format!("{} has unsaved changes", window_title).as_str(),
                            Some("What do you want to do with the changes?"),
                            &[
                                PromptButton::ok("Save"),
                                PromptButton::ok("Discard"),
                                PromptButton::cancel("Keep Editing"),
                            ],
                            cx,
                        )
                    })
                    .expect("should update window");

                cx.spawn(async move |cx| {
                    let Ok(ix) = answer.await else { return };

                    singleton_window
                        .handle
                        .update(cx, move |view, window, cx| {
                            let wrapper: Entity<WindowWrapper<D>> = view
                                .view()
                                .clone()
                                .downcast()
                                .expect("Root view should be a WindowWrapper");

                            match ix {
                                0 => {
                                    wrapper.update(cx, |wrapper, cx| {
                                        wrapper.handle_window_save(window, cx);
                                    });
                                    close_window(cx);
                                }
                                1 => {
                                    wrapper.update(cx, |wrapper, cx| {
                                        wrapper.handle_window_discard(window, cx);
                                    });
                                    close_window(cx);
                                }
                                2 => {}
                                _ => {}
                            }
                        })
                        .expect("should update window");
                })
                .detach();
            });
        }

        false
    }

    // Dock windows
    pub fn add_dock_window(&mut self, config: DockWindowConfig, cx: &mut App) {
        let window_handle = cx
            .open_window(config.gpui_window_options(), |window, cx| {
                window.set_window_title("demex");

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
