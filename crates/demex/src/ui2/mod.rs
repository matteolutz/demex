use demex_ui::wm::WindowDelegate;
use gpui::{App, Context, Entity, Window, prelude::*};

pub mod pane;
pub mod window;

use crate::ui2::pane::layout::LayoutViewPane;

pub struct MainWindow {
    pane: Entity<LayoutViewPane>,
}

impl WindowDelegate for MainWindow {
    type InitData = ();

    fn create(window: &mut Window, cx: &mut App, _: Self::InitData) -> Self
    where
        Self: Sized,
    {
        window.set_app_id("demex");
        window.set_window_title("demex");

        Self {
            pane: cx.new(|cx| LayoutViewPane::new(window, cx)),
        }
    }

    fn render_content(
        &mut self,
        _window: &mut Window,
        _cx: &mut Context<demex_ui::wm::WindowWrapper<Self>>,
    ) -> impl IntoElement
    where
        Self: Sized,
    {
        self.pane.clone()
    }
}
