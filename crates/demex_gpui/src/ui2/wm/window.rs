use std::ops::{Deref, DerefMut};

use gpui::{
    App, FontWeight, Pixels, TitlebarOptions, Window, WindowControlArea, WindowHandle,
    WindowOptions, div, point, px,
};
use gpui::{WindowBounds, prelude::*};
use gpui_component::ActiveTheme;

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

    fn render_content(
        &mut self,
        window: &mut Window,
        cx: &mut Context<WindowWrapper<Self>>,
    ) -> impl IntoElement
    where
        Self: Sized;
}

pub struct WindowWrapper<D: WindowDelegate> {
    delegate: D,
    window_handle: WindowHandle<Self>,
}

impl<D: WindowDelegate> WindowWrapper<D> {
    pub fn open<F: FnOnce(&mut Window, &mut App) -> D>(cx: &mut App, f: F) -> WindowHandle<Self> {
        let window_bounds = D::window_bounds(cx);

        cx.open_window(window_options(window_bounds), |window, cx| {
            let delegate = f(window, cx);
            cx.new(|_| Self {
                delegate,
                window_handle: window.window_handle().downcast().unwrap(),
            })
        })
        .expect("should open window")
    }

    pub fn window_handle(&self) -> WindowHandle<Self> {
        self.window_handle.clone()
    }
}

impl<D: WindowDelegate> Render for WindowWrapper<D> {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .flex_col()
            .font_family("Inter 18pt")
            .text_color(cx.theme().foreground)
            .text_sm()
            .bg(cx.theme().background)
            .child(render_titlebar(window, cx))
            .child(self.render_content(window, cx))
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

fn render_titlebar(window: &Window, cx: &App) -> impl IntoElement {
    let titlebar_height = px(32.0);

    div()
        .id("titlebar")
        .window_control_area(WindowControlArea::Drag)
        .w_full()
        .min_h(titlebar_height)
        .max_h(titlebar_height)
        .pl(TRAFFIC_LIGHT_WIDTH * 3 + TRAFFIC_LIGHT_SPACING * 4)
        .pr(TRAFFIC_LIGHT_SPACING)
        .border_b_1()
        .border_color(cx.theme().title_bar_border)
        .bg(cx.theme().title_bar)
        .flex()
        .items_center()
        .child(
            div()
                .font_weight(FontWeight::BOLD)
                .text_color(cx.theme().foreground.opacity(0.8))
                .child(window.window_title()),
        )
        .on_click(|event, window, _| {
            if event.click_count() == 2 {
                window.titlebar_double_click();
            }
        })
}
