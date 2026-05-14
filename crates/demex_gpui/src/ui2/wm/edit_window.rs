use std::rc::Rc;

use gpui::{
    AnyView, App, AppContext, Context, Entity, ParentElement, Render, SharedString, Styled, Window,
    WindowBounds, WindowKind,
};
use gpui_component::v_flex;

use crate::ui2::{
    titlebar::DemexTitleBar,
    wm::{
        WindowManager,
        app::WindowManagerAppExt,
        window::{WindowDelegate, WindowWrapper},
    },
};

pub trait EditWindowDelegate: 'static + Render {
    fn window_title(&self, window: &mut Window, cx: &App) -> impl Into<SharedString>;

    fn should_have_save_button(_cx: &App) -> bool
    where
        Self: Sized,
    {
        true
    }

    fn handle_save(&self, window: &mut Window, cx: &mut App);
    fn handle_discard(&self, window: &mut Window, cx: &mut App);

    fn window_bounds(_cx: &mut App) -> Option<WindowBounds> {
        None
    }

    fn set_edited(&self, edited: bool, cx: &mut App) {
        cx.update_wm(|wm, cx| wm.set_singleton_window_edited::<EditWindow<Self>>(cx, edited))
    }

    fn close(&self, cx: &mut App)
    where
        Self: Sized,
    {
        cx.update_wm(|wm, cx| {
            wm.request_close_singleton_window::<EditWindow<Self>>(cx, false, false)
        });
    }

    fn discard_and_close(&self, cx: &mut App)
    where
        Self: Sized,
    {
        cx.update_wm(|wm, cx| {
            wm.request_close_singleton_window::<EditWindow<Self>>(cx, true, false)
        });
    }

    /// Whether the window should be reactivated when being reopened.
    /// When the window has some sort of InitData this should return false
    /// to ensure the window is not reactivated when being opened with different data.
    fn should_reactivate() -> bool {
        false
    }

    fn is_edited(&self, cx: &App) -> bool {
        cx.wm().is_singleton_window_edited::<EditWindow<Self>>()
    }
}

pub struct EditWindowView<V: EditWindowDelegate> {
    pub entity: Entity<V>,
    pub titlebar: Entity<DemexTitleBar>,
}

impl<V: EditWindowDelegate> Render for EditWindowView<V> {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        v_flex()
            .size_full()
            .child(self.titlebar.clone())
            .child(self.entity.clone())
    }
}

pub struct EditWindow<V: EditWindowDelegate> {
    entity: Entity<V>,
    view: Entity<EditWindowView<V>>,
}

impl<V: EditWindowDelegate> WindowDelegate for EditWindow<V> {
    type InitData = Rc<dyn Fn(&mut Window, &mut Context<V>) -> V>;

    fn create(window: &mut Window, cx: &mut App, data: Self::InitData) -> Self
    where
        Self: Sized,
    {
        let data = cx.new(|cx| data(window, cx));
        let window_title = data.read(cx).window_title(window, cx).into();

        Self {
            entity: data.clone(),
            view: cx.new(|cx| EditWindowView {
                entity: data,
                titlebar: cx.new(|cx| DemexTitleBar::settings(window_title, cx)),
            }),
        }
    }

    fn window_title(&self, window: &mut Window, cx: &App) -> impl Into<SharedString>
    where
        Self: Sized,
    {
        self.entity.read_with(cx, |d, cx| {
            let mut title = d.window_title(window, cx).into().to_string();

            if d.is_edited(cx) {
                title += " *";
            }

            SharedString::from(title)
        })
    }

    fn should_have_save_button(cx: &App) -> bool
    where
        Self: Sized,
    {
        V::should_have_save_button(cx)
    }

    fn handle_window_save(&self, window: &mut Window, cx: &mut Context<WindowWrapper<Self>>)
    where
        Self: Sized,
    {
        self.entity.update(cx, |d, cx| d.handle_save(window, cx))
    }

    fn handle_window_discard(&self, window: &mut Window, cx: &mut Context<WindowWrapper<Self>>)
    where
        Self: Sized,
    {
        self.entity.update(cx, |d, cx| d.handle_discard(window, cx))
    }

    fn window_bounds(cx: &mut App) -> Option<WindowBounds> {
        V::window_bounds(cx)
    }

    fn window_kind(_cx: &mut App) -> WindowKind {
        #[cfg(target_os = "windows")]
        return WindowKind::Floating;

        #[cfg(target_os = "macos")]
        return WindowKind::PopUp;

        #[cfg(target_os = "linux")]
        return WindowKind::Floating;
    }

    fn matches_data(&self, _data: &Self::InitData) -> bool {
        V::should_reactivate()
    }

    fn view(&self) -> AnyView
    where
        Self: Sized,
    {
        self.view.clone().into()
    }
}

pub trait WindowManagerExtension<V: EditWindowDelegate> {
    fn open_edit_window<D: EditWindowDelegate>(
        cx: &mut App,
        data: impl Fn(&mut Window, &mut Context<V>) -> V + 'static,
    );
}

impl<V: EditWindowDelegate> WindowManagerExtension<V> for WindowManager {
    fn open_edit_window<D: EditWindowDelegate>(
        cx: &mut App,
        data: impl Fn(&mut Window, &mut Context<V>) -> V + 'static,
    ) {
        Self::open_singleton_window::<EditWindow<V>>(cx, Rc::new(data));
    }
}
