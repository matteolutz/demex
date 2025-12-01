use gpui::{App, Context, Entity, Window, prelude::*};

pub mod assets;
pub mod config;
pub mod ext;
pub mod pane;
pub mod titlebar;
pub mod utils;
pub mod window;
pub mod wm;

use crate::ui2::{
    pane::MainPane,
    wm::{WindowDelegate, WindowWrapper},
};

pub fn init(cx: &mut App) -> gpui::Result<()> {
    assets::init(cx)?;
    Ok(())
}

pub struct MainWindow {
    pane: Entity<MainPane>,
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
            pane: cx.new(|cx| MainPane::new(window, cx)),
        }
    }

    fn render_content(
        &mut self,
        _window: &mut Window,
        _cx: &mut Context<WindowWrapper<Self>>,
    ) -> impl IntoElement
    where
        Self: Sized,
    {
        self.pane.clone()
    }
}
