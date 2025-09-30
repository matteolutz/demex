use demex_ui::fader::Fader;
use gpui::{App, AppContext, Entity, ParentElement, Render, Styled, Window, div};

use crate::engine::DemexEngineHandler;

pub struct Playback {
    fader: Entity<Fader>,
    executor_id: u32,
}

impl Playback {
    pub fn new(window: &mut Window, cx: &mut App, executor_id: u32) -> Self {
        let fader_value = cx.new(|cx| {
            DemexEngineHandler::engine(cx)
                .updatable_handler()
                .read(|uh| {
                    uh.executor(executor_id)
                        .ok()
                        .map(|executor| executor.value())
                        .unwrap_or(0.0)
                })
        });

        cx.observe(&fader_value, move |_value, _cx| {
            log::debug!("update executor with id: {}", executor_id);
        })
        .detach();

        Self {
            fader: cx.new(|cx| Fader::new(fader_value, window, cx)),
            executor_id,
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
