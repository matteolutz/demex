use gpui::{App, Bounds, ParentElement, Point, WindowBounds, div, px};

use crate::ui2::wm::{WindowDelegate, WindowWrapper};

pub struct AddLayoutItemWindowInitData {
    pub selection: (Point<u16>, Point<u16>),
}

pub struct AddLayoutItemWindow {
    data: AddLayoutItemWindowInitData,
}

impl WindowDelegate for AddLayoutItemWindow {
    type InitData = AddLayoutItemWindowInitData;

    fn create(_window: &mut gpui::Window, _cx: &mut gpui::App, data: Self::InitData) -> Self
    where
        Self: Sized,
    {
        Self { data }
    }

    fn window_bounds(cx: &mut App) -> Option<gpui::WindowBounds> {
        Some(WindowBounds::Windowed(Bounds::centered(
            None,
            gpui::size(px(200.0), px(200.0)),
            cx,
        )))
    }

    fn render_content(
        &mut self,
        _window: &mut gpui::Window,
        cx: &mut gpui::Context<WindowWrapper<Self>>,
    ) -> impl gpui::IntoElement
    where
        Self: Sized,
    {
        div().child(format!("{:?}", self.data.selection))
    }
}
