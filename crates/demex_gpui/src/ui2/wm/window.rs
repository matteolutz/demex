use std::ops::{Deref, DerefMut};

use gpui::{AnyView, App, SharedString, Window, WindowHandle, WindowKind, WindowOptions};
use gpui::{WindowBounds, prelude::*};
use gpui_component::{Root, v_flex};

use crate::ui2::titlebar::titlebar_options;
use crate::ui2::wm::DEMEX_APP_ID;
use crate::ui2::wm::app::WindowManagerAppExt;

pub trait WindowDelegate: 'static {
    type InitData: 'static;

    fn create(window: &mut Window, cx: &mut App, data: Self::InitData) -> Self
    where
        Self: Sized;

    fn should_have_save_button(_cx: &App) -> bool
    where
        Self: Sized,
    {
        true
    }

    fn handle_window_save(&self, _window: &mut Window, _cx: &mut Context<WindowWrapper<Self>>)
    where
        Self: Sized,
    {
    }

    fn handle_window_discard(&self, _window: &mut Window, _cx: &mut Context<WindowWrapper<Self>>)
    where
        Self: Sized,
    {
    }

    fn window_bounds(_cx: &mut App) -> Option<WindowBounds> {
        None
    }

    fn window_kind(_cx: &mut App) -> WindowKind {
        WindowKind::Normal
    }

    fn set_edited(&self, edited: bool, cx: &mut App)
    where
        Self: Sized,
    {
        cx.update_wm(|wm, cx| wm.set_singleton_window_edited::<Self>(cx, edited));
    }

    fn close(&self, cx: &mut App, discard_changes: bool)
    where
        Self: Sized,
    {
        cx.update_wm(|wm, cx| {
            wm.request_close_singleton_window::<Self>(cx, discard_changes, false)
        });
    }

    fn window_title(&self, window: &mut Window, cx: &App) -> impl Into<SharedString>
    where
        Self: Sized;

    fn view(&self) -> AnyView
    where
        Self: Sized;
}

pub struct WindowWrapper<D: WindowDelegate> {
    delegate: D,
}

impl<D: WindowDelegate> WindowWrapper<D> {
    pub fn open<F>(cx: &mut App, f: F) -> WindowHandle<Root>
    where
        F: FnOnce(&mut Window, &mut App) -> D,
    {
        let window_bounds = D::window_bounds(cx);
        let window_kind = D::window_kind(cx);

        cx.open_window(
            singleton_window_options(window_bounds, window_kind),
            |window, cx| {
                let delegate = f(window, cx);

                let window_title = delegate.window_title(window, cx).into();
                window.set_window_title(window_title.as_str());

                cx.new(|cx| Root::new(cx.new(|_| Self { delegate }), window, cx))
            },
        )
        .expect("should open window")
    }
}

impl<D: WindowDelegate> Render for WindowWrapper<D> {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .relative()
            .size_full()
            .child(self.delegate.view())
            .children(Root::render_dialog_layer(window, cx))
            .children(Root::render_sheet_layer(window, cx))
            .children(Root::render_notification_layer(window, cx))
    }
}

impl<D: WindowDelegate> Deref for WindowWrapper<D> {
    type Target = D;

    fn deref(&self) -> &Self::Target {
        &self.delegate
    }
}

impl<D: WindowDelegate> DerefMut for WindowWrapper<D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.delegate
    }
}

fn singleton_window_options(
    window_bounds: Option<WindowBounds>,
    kind: WindowKind,
) -> WindowOptions {
    WindowOptions {
        window_bounds,
        kind,
        titlebar: Some(titlebar_options()),
        app_id: Some(DEMEX_APP_ID.to_string()),
        ..Default::default()
    }
}
