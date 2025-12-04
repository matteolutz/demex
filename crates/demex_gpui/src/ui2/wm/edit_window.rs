use gpui::{
    AnyView, App, AppContext, Context, Entity, Render, SharedString, Window, WindowBounds,
    WindowKind,
};

use crate::ui2::wm::{
    WindowManager,
    app::WindowManagerAppExt,
    window::{WindowDelegate, WindowWrapper},
};

pub trait EditWindowDelegate: 'static + Render {
    fn window_title(&self, window: &mut Window, cx: &App) -> impl Into<SharedString>;

    fn handle_save(&self, window: &mut Window, cx: &mut App);
    fn handle_discard(&self, window: &mut Window, cx: &mut App);

    fn window_bounds(_cx: &mut App) -> Option<WindowBounds> {
        None
    }

    fn set_edited(&self, edited: bool, cx: &mut App) {
        cx.update_wm(|wm, cx| wm.set_singleton_window_edited::<EditWindow<Self>>(cx, edited))
    }

    fn is_edited(&self, cx: &App) -> bool {
        cx.wm().is_singleton_window_edited::<EditWindow<Self>>()
    }
}

pub struct EditWindow<V: EditWindowDelegate> {
    view: Entity<V>,
}

impl<V: EditWindowDelegate> WindowDelegate for EditWindow<V> {
    type InitData = V;

    fn create(
        _window: &mut Window,
        cx: &mut App,
        data: impl FnOnce(&mut Context<Self::InitData>) -> Self::InitData,
    ) -> Self
    where
        Self: Sized,
    {
        Self {
            view: cx.new(|cx| data(cx)),
        }
    }

    fn window_title(&self, window: &mut Window, cx: &App) -> impl Into<SharedString>
    where
        Self: Sized,
    {
        self.view
            .read_with(cx, |d, cx| d.window_title(window, cx).into())
    }

    fn handle_window_save(&self, window: &mut Window, cx: &mut Context<WindowWrapper<Self>>)
    where
        Self: Sized,
    {
        self.view.update(cx, |d, cx| d.handle_save(window, cx))
    }

    fn handle_window_discard(&self, window: &mut Window, cx: &mut Context<WindowWrapper<Self>>)
    where
        Self: Sized,
    {
        self.view.update(cx, |d, cx| d.handle_discard(window, cx))
    }

    fn window_bounds(cx: &mut App) -> Option<WindowBounds> {
        V::window_bounds(cx)
    }

    fn window_kind(_cx: &mut App) -> WindowKind {
        WindowKind::PopUp
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
        &mut self,
        cx: &mut App,
        data: impl FnOnce(&mut Context<V>) -> V,
    );
}

impl<V: EditWindowDelegate> WindowManagerExtension<V> for WindowManager {
    fn open_edit_window<D: EditWindowDelegate>(
        &mut self,
        cx: &mut App,
        data: impl FnOnce(&mut Context<V>) -> V,
    ) {
        self.open_singleton_window::<EditWindow<V>>(cx, data);
    }
}
