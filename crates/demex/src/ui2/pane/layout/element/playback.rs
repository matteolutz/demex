use demex_ui::fader::Fader;
use gpui::{App, AppContext, Entity, IntoElement, ParentElement, Render, Styled, Window, div};

pub struct Playback {
    fader: Entity<Fader>,
}

impl Playback {
    pub fn new(window: &mut Window, cx: &mut App) -> Self {
        Self {
            fader: cx.new(|cx| Fader::new(window, cx)),
        }
    }
}

impl Render for Playback {
    fn render(
        &mut self,
        window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        div().size_full().child(self.fader.clone())
    }
}
