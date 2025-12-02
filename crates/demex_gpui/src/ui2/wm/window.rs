use std::ops::{Deref, DerefMut};

use gpui::{AnyView, App, Pixels, TitlebarOptions, Window, WindowHandle, WindowOptions, point, px};
use gpui::{WindowBounds, prelude::*};
use gpui_component::Root;

pub const TRAFFIC_LIGHT_WIDTH: Pixels = px(14.0);
pub const TRAFFIC_LIGHT_SPACING: Pixels = px(9.0);

pub trait WindowDelegate: 'static {
    type InitData: 'static + Send;

    fn create(window: &mut Window, cx: &mut App, data: Self::InitData) -> Self
    where
        Self: Sized;

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

    fn view(&self) -> AnyView
    where
        Self: Sized;
}

pub struct WindowWrapper<D: WindowDelegate> {
    delegate: D,
    window_handle: WindowHandle<Self>,
}

impl<D: WindowDelegate> WindowWrapper<D> {
    pub fn open<F: FnOnce(&mut Window, &mut App) -> D>(cx: &mut App, f: F) -> WindowHandle<Root> {
        let window_bounds = D::window_bounds(cx);

        cx.open_window(window_options(window_bounds), |window, cx| {
            let delegate = f(window, cx);
            cx.new(|cx| Root::new(delegate.view(), window, cx))
        })
        .expect("should open window")
    }

    pub fn window_handle(&self) -> WindowHandle<Self> {
        self.window_handle.clone()
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

pub fn window_options(window_bounds: Option<WindowBounds>) -> WindowOptions {
    WindowOptions {
        window_bounds,
        titlebar: Some(TitlebarOptions {
            appears_transparent: true,
            traffic_light_position: Some(point(TRAFFIC_LIGHT_SPACING, TRAFFIC_LIGHT_SPACING)),
            ..Default::default()
        }),
        ..Default::default()
    }
}
