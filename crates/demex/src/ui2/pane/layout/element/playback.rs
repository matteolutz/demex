use demex_core::command::parser::nodes::action::Action;
use demex_core::command::parser::nodes::object::Object;
use demex_core::event::DemexEvent;
use demex_ui::fader::Fader;
use gpui::{App, AppContext, Context, Entity, ParentElement, Render, Styled, Window, div};

use crate::engine::DemexEngineHandler;
use crate::ui2::ext::GpuiContextExtension;

pub struct Playback {
    fader: Entity<Fader>,
    executor_id: u32,
}

impl Playback {
    pub fn new(window: &mut Window, cx: &mut Context<Self>, executor_id: u32) -> Self {
        let get_fader_value = move |cx: &mut App| {
            DemexEngineHandler::engine(cx)
                .updatable_handler()
                .read(|uh| {
                    uh.executor(executor_id)
                        .ok()
                        .map(|executor| executor.value())
                        .unwrap_or(0.0)
                })
        };

        let fader_value = cx.new(|cx| get_fader_value(cx));

        cx.observe(&fader_value, move |_, value, cx| {
            let value = *value.read(cx);

            DemexEngineHandler::engine(cx)
                .exec_ui(Action::InternalExecutorSetFaderValue(executor_id, value));
        })
        .detach();

        let event_handler = DemexEngineHandler::event_handler(cx);

        cx.subscribe_with_entity(
            &event_handler,
            fader_value.clone(),
            move |this, _, event, fader_value, cx| match event {
                DemexEvent::ExecutorFaderValueChanged(event_executor_id)
                    if *event_executor_id == executor_id =>
                {
                    cx.update_entity(&fader_value.clone(), |fader_value, cx| {
                        *fader_value = get_fader_value(cx);
                        // dont notify, this would cause an infinite loop
                    })
                }
                DemexEvent::ObjectPropertyChanged(object, _)
                    if matches!(object, Object::Sequence(sequence_id) if this.get_sequence_id(cx).is_some_and(|id| id == *sequence_id)) => {
                    cx.notify();
                }
                _ => {}
            },
        )
        .detach();

        Self {
            fader: cx.new(|cx| Fader::new(fader_value, window, cx)),
            executor_id,
        }
    }

    fn get_sequence_id(&self, cx: &mut App) -> Option<u32> {
        DemexEngineHandler::engine(cx)
            .updatable_handler()
            .read(|uh| {
                uh.executor(self.executor_id)
                    .ok()
                    .map(|e| e.runtime().sequence_id())
            })
    }
}

impl Render for Playback {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        let sequence_id = self.get_sequence_id(cx);

        let sequence_name = sequence_id
            .and_then(|sequence_id| {
                DemexEngineHandler::engine(cx).preset_handler().read(|ph| {
                    ph.get_sequence(sequence_id)
                        .ok()
                        .map(|sequence| sequence.name().to_string())
                })
            })
            .unwrap_or_else(|| "(deleted sequence)".to_string());

        div()
            .size_full()
            .flex()
            .flex_col()
            .child(div().w_full().p_2().child(sequence_name))
            .child(self.fader.clone())
    }
}
