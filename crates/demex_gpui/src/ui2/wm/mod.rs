use std::{any::TypeId, collections::HashMap};

use gpui::{
    AnyWindowHandle, App, AppContext, Context, Entity, Global, PromptButton, PromptLevel,
    SharedString, Window, WindowHandle,
};
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
            let dock_window_closed = cx.update_wm(|wm, cx| {
                wm.dock_windows
                    .iter()
                    // why ever this thing needs &mut App??
                    .any(|h| h.is_active(cx).is_none())
            });

            // check if singleton windows are open
            // on_window_should_close is not being called on X11
            cx.update_wm(|wm, cx| {
                wm.singleton_windows
                    .retain(|_, w| w.handle.is_active(cx).is_some());
            });

            if dock_window_closed && cx.wm().auto_quit {
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

    /// When set to true, the application is quit when any dock ("main") window is closed
    pub fn auto_quit(mut self, auto_quit: bool) -> Self {
        self.auto_quit = auto_quit;
        self
    }

    // Singleton windows
    pub fn open_singleton_window<D: WindowDelegate>(cx: &mut App, data: D::InitData) {
        let type_id = TypeId::of::<D>();

        if cx.wm().singleton_windows.contains_key(&type_id) {
            cx.update_wm(|wm, cx| {
                let _ = wm
                    .singleton_windows
                    .get(&type_id)
                    .unwrap()
                    .handle
                    .update(cx, |_, window, _| window.activate_window());
            });
            return;
        }

        let handle = WindowWrapper::open(cx, |window, cx| {
            window.on_window_should_close(cx, move |_, cx| {
                cx.update_wm(|wm, cx| wm.request_close_singleton_window::<D>(cx, false, true))
            });

            D::create(window, cx, data)
        });

        cx.update_wm(|wm, _| {
            wm.singleton_windows
                .insert(type_id, SingletonWindow::new(handle))
        });
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

    pub fn is_singleton_window_edited<D: WindowDelegate>(&self) -> bool {
        let type_id = TypeId::of::<D>();
        self.singleton_windows
            .get(&type_id)
            .map(|sw| sw.is_edited)
            .unwrap_or(false)
    }

    pub fn request_close_singleton_window<D: WindowDelegate>(
        &mut self,
        cx: &mut App,
        discard_changes: bool,
        closes_on_false: bool,
    ) -> bool {
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

        if !singleton_window.is_edited || discard_changes {
            if closes_on_false {
                self.singleton_windows.remove(&type_id);
            } else {
                close_window(cx);
            }

            return true;
        }

        if singleton_window.is_edited {
            let display_save_button = D::should_have_save_button(cx);

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

                        let prompt_buttons_save = &[
                            PromptButton::ok("Save"),
                            PromptButton::new("Discard"),
                            PromptButton::cancel("Keep Editing"),
                        ];
                        let prompt_buttons = &[
                            PromptButton::new("Discard"),
                            PromptButton::cancel("Keep Editing"),
                        ];

                        window.prompt(
                            PromptLevel::Warning,
                            format!("{} has unsaved changes", window_title).as_str(),
                            Some("What do you want to do with the changes?"),
                            if display_save_button {
                                prompt_buttons_save
                            } else {
                                prompt_buttons
                            },
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
                                0 if display_save_button => {
                                    wrapper.update(cx, |wrapper, cx| {
                                        wrapper.handle_window_save(window, cx);
                                    });
                                    close_window(cx);
                                }
                                0 if !display_save_button => {
                                    wrapper.update(cx, |wrapper, cx| {
                                        wrapper.handle_window_discard(window, cx);
                                    });
                                    close_window(cx);
                                }
                                1 if display_save_button => {
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
    pub fn add_dock_window(config: Option<DockWindowConfig>, cx: &mut App) {
        let window_handle = cx
            .open_window(DockWindowConfig::gpui_window_options(), |window, cx| {
                window.set_window_title("demex");

                cx.new(|cx| Root::new(cx.new(|cx| DockWindow::new(config, window, cx)), window, cx))
            })
            .expect("Dock window should be opened");

        cx.update_wm(|wm, _| wm.dock_windows.push(window_handle))
    }

    pub fn add_dock_windows(
        configs: impl IntoIterator<Item = Option<DockWindowConfig>>,
        cx: &mut App,
    ) {
        configs
            .into_iter()
            .for_each(|config| Self::add_dock_window(config, cx));
    }

    pub fn reset_dock_window_configs(&mut self, cx: &mut App) {
        for handle in &self.dock_windows {
            let _ = Self::update_dock_window(
                handle,
                |dock_window, window, cx| {
                    dock_window.reset_config(window, cx);
                },
                cx,
            );
        }
    }

    pub fn update_dock_window_configs(
        &mut self,
        configs: impl IntoIterator<Item = DockWindowConfig>,
        cx: &mut App,
    ) {
        let mut configs = configs.into_iter();

        for handle in &self.dock_windows {
            let config = configs.next();

            let _ = Self::update_dock_window(
                handle,
                |dock_window, window, cx| {
                    if let Some(config) = config {
                        dock_window.update_config(config.dock_area_state, window, cx);
                    } else {
                        dock_window.reset_config(window, cx);
                    }
                },
                cx,
            );
        }
    }

    fn dock_window_entity(handle: &WindowHandle<Root>, cx: &App) -> Entity<DockWindow> {
        handle
            .read(cx)
            .expect("Should read window Root")
            .view()
            .clone()
            .downcast::<DockWindow>()
            .expect("Root view should be a DockWindow")
    }

    fn read_dock_window<'a>(handle: &'a WindowHandle<Root>, cx: &'a App) -> &'a DockWindow {
        Self::dock_window_entity(handle, cx).read(cx)
    }

    fn update_dock_window<'a, R>(
        handle: &'a WindowHandle<Root>,
        update: impl FnOnce(&mut DockWindow, &mut Window, &mut Context<DockWindow>) -> R,
        cx: &'a mut App,
    ) -> gpui::Result<R> {
        let dock_window = Self::dock_window_entity(handle, cx);
        handle.update(cx, |_, window, cx| {
            dock_window.update(cx, |dock_window, cx| update(dock_window, window, cx))
        })
    }

    pub fn main_dock_window<'a>(&'a self, cx: &'a App) -> (&'a WindowHandle<Root>, &'a DockWindow) {
        let handle = &self.dock_windows[0];
        (handle, Self::read_dock_window(handle, cx))
    }

    pub fn dock_window_for<'a>(window: AnyWindowHandle, cx: &'a App) -> Option<Entity<DockWindow>> {
        let window_handle: WindowHandle<Root> = window.downcast()?;
        Some(Self::dock_window_entity(&window_handle, cx))
    }

    pub fn main_dock_window_handle_mut(&mut self) -> &mut WindowHandle<Root> {
        &mut self.dock_windows[0]
    }

    pub fn update_main_dock_window<'a, R>(
        &mut self,
        cx: &'a mut App,
        update: impl FnOnce(&mut DockWindow, &mut Window, &mut Context<DockWindow>) -> R,
    ) -> gpui::Result<R> {
        let handle = &self.dock_windows[0];
        Self::update_dock_window(handle, update, cx)
    }

    pub fn read_dock_window_handle<'a>(handle: AnyWindowHandle, cx: &'a App) -> &'a DockWindow {
        let handle = handle.downcast::<Root>().unwrap();
        Self::dock_window_entity(&handle, cx).read(cx)
    }

    pub fn update_dock_window_handle<'a, R>(
        window_handle: AnyWindowHandle,
        cx: &'a mut App,
        update: impl FnOnce(&mut DockWindow, &mut Window, &mut Context<DockWindow>) -> R,
    ) -> gpui::Result<R> {
        let handle = window_handle.downcast::<Root>().unwrap();
        Self::update_dock_window(&handle, update, cx)
    }

    pub fn dock_windows<'a>(
        &'a self,
        cx: &'a App,
    ) -> impl Iterator<Item = (&'a WindowHandle<Root>, &'a DockWindow)> {
        self.dock_windows
            .iter()
            .map(|handle| (handle, Self::read_dock_window(handle, cx)))
    }

    pub fn dock_window_configs<'a>(
        &'a self,
        cx: &'a App,
    ) -> impl Iterator<Item = DockWindowConfig> {
        self.dock_windows(cx)
            .map(|(_, window)| window.dump_config(cx))
    }

    pub fn focus_panel(&mut self, panel_name: &str, cx: &mut App) -> bool {
        self.dock_windows
            .iter_mut()
            .find(|dw| {
                dw.update(cx, |root, window, cx| {
                    root.view()
                        .clone()
                        .downcast::<DockWindow>()
                        .expect("Root view should be a DockWindow")
                        .update(cx, |dw, cx| dw.focus_panel(panel_name, window, cx))
                })
                .unwrap()
            })
            .is_some()
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

    pub fn push_success(&mut self, success: impl Into<SharedString>, cx: &mut App) {
        self.push_notifcation(Notification::success(success), cx);
    }

    pub fn push_error(&mut self, error: impl Into<SharedString>, cx: &mut App) {
        self.push_notifcation(Notification::error(error), cx);
    }

    pub fn inspect_error<R, E: std::fmt::Display>(
        result: Result<R, E>,
        prefix: &str,
        cx: &mut App,
    ) -> Result<R, E> {
        result.inspect_err(|err| {
            let err = format!("{}{}", prefix, err);
            cx.defer(|cx| cx.update_wm(|wm, cx| wm.push_error(err, cx)));
        })
    }

    pub fn handle_error<E: std::fmt::Display>(prefix: &str) -> impl Fn(&E, &mut App) {
        move |err, cx| {
            let err = format!("{}{}", prefix, err);
            cx.defer(|cx| cx.update_wm(|wm, cx| wm.push_error(err, cx)));
        }
    }
}

impl Global for WindowManager {}
