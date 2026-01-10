use demex_core::{
    command::parser::nodes::object::Object,
    engine::comm::{SequenceRequest, SequenceResponse},
    event::{DemexEvent, DemexExecutorUpdateEvent},
    sequence::SequenceProperty,
};
use gpui::{
    App, AppContext, Context, Entity, EventEmitter, FocusHandle, Focusable, IntoElement,
    ParentElement, Render, Styled, Subscription, Window,
};
use gpui_component::{
    ActiveTheme, Sizable, StyledExt,
    button::{Button, ButtonVariants},
    dock::{Panel, PanelEvent, register_panel},
    h_flex,
    scroll::ScrollableElement,
    table::{Table, TableState},
    v_flex,
};

use crate::{
    engine::{DemexEngineHandler, state::DemexUiState},
    ui2::{
        config::AppConfigExt,
        panels::{sequence_editor::table::SequenceEditorTable, toolbar_buttons},
        window::set_property::{SetPropertyWindow, SetPropertyWindowPropertyType},
        wm::{WindowManager, edit_window::WindowManagerExtension},
    },
};

mod table;

const SEQUENCE_EDITOR_PANEL_NAME: &str = "demex-sequence-editor";

pub(super) fn register(cx: &mut App) {
    register_panel(cx, SEQUENCE_EDITOR_PANEL_NAME, |_, _, _, window, cx| {
        Box::new(cx.new(|cx| SequenceEditorPanel::new(window, cx)))
    });
}

pub struct SequenceEditorPanel {
    focus_handle: FocusHandle,

    sequence: Entity<Option<SequenceResponse>>,
    table_state: Entity<TableState<SequenceEditorTable>>,

    _subscriptions: Vec<Subscription>,
}

impl SequenceEditorPanel {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let table_state = cx.new(|cx| {
            TableState::new(SequenceEditorTable::new(None), window, cx).col_movable(false)
        });

        let sequence: Entity<Option<SequenceResponse>> = cx.new(|_| None);

        let _subscriptions = vec![
            cx.observe(&DemexUiState::selected_sequence(cx), |this, _, cx| {
                this.request_sequence(cx);
            }),
            cx.observe(&sequence, |this, sequence, cx| {
                let data = sequence
                    .read(cx)
                    .as_ref()
                    .map(|seq| (seq.sequence.id, seq.sequence.cues.clone()));

                let active_cues = sequence
                    .read(cx)
                    .as_ref()
                    .and_then(|seq| seq.first_executor.as_ref())
                    .map(|(_, cues)| cues.clone());

                this.table_state.update(cx, |table, cx| {
                    table.delegate_mut().update_data(data);
                    table.delegate_mut().update_active_cues(active_cues);
                    cx.notify();
                });
                cx.notify();
            }),
            cx.subscribe(
                &DemexEngineHandler::event_handler(cx),
                |this, _, evt, cx| {
                    let sequence_id = this.sequence.read(cx).as_ref().map(|seq| seq.sequence.id);

                    let Some(sequence_id) = sequence_id else {
                        return;
                    };

                    match evt {
                        DemexEvent::ObjectPropertyChanged(obj, _)
                            if matches!(obj,
                                &Object::SequenceCue(cue_seq_id, _) if cue_seq_id == sequence_id
                            ) || matches!(obj, &Object::Sequence(id) if id == sequence_id) =>
                        {
                            this.request_sequence(cx);
                        }
                        _ => {}
                    }

                    let executor_id = this
                        .sequence
                        .read(cx)
                        .as_ref()
                        .and_then(|seq| seq.first_executor.as_ref())
                        .map(|(id, _)| *id);

                    let Some(executor_id) = executor_id else {
                        return;
                    };

                    match evt {
                        DemexEvent::ExecutorStop(id) if *id == executor_id => {
                            this.table_state.update(cx, |state, cx| {
                                state.delegate_mut().executor_stop();
                                cx.notify();
                            });
                        }
                        DemexEvent::ExecutorUpdateEvent { id, event } if *id == executor_id => {
                            log::debug!("got executor update event: {:?}", event);
                            match event {
                                DemexExecutorUpdateEvent::CueActivate(cue_ix, at) => {
                                    this.table_state.update(cx, |state, cx| {
                                        state.delegate_mut().cue_activated(*cue_ix, *at);
                                        cx.notify();
                                        // state.refresh(cx);
                                    });
                                    // cx.notify();
                                }
                                DemexExecutorUpdateEvent::CueDeactivate(cue_ix) => {
                                    this.table_state.update(cx, |state, cx| {
                                        state.delegate_mut().cue_deactivated(cue_ix);
                                        cx.notify();
                                    });
                                }
                            }
                        }
                        _ => {}
                    }
                },
            ),
        ];

        let s = Self {
            focus_handle: cx.focus_handle(),
            sequence,
            table_state,
            _subscriptions,
        };

        s.request_sequence(cx);

        s
    }

    pub fn request_sequence(&self, cx: &mut Context<Self>) {
        let Some(sequence_id) = *DemexUiState::selected_sequence(cx).read(cx) else {
            self.sequence.update(cx, |seq, cx| {
                *seq = None;
                cx.notify();
            });
            return;
        };

        DemexEngineHandler::send_with(
            cx,
            self.sequence.clone(),
            SequenceRequest { sequence_id },
            |engine_seq, seq, cx| {
                *seq = engine_seq;
                cx.notify();
            },
        );
    }
}

impl EventEmitter<PanelEvent> for SequenceEditorPanel {}
impl Focusable for SequenceEditorPanel {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Panel for SequenceEditorPanel {
    fn panel_name(&self) -> &'static str {
        SEQUENCE_EDITOR_PANEL_NAME
    }

    fn title(&mut self, _window: &mut gpui::Window, _cx: &mut Context<Self>) -> impl IntoElement {
        "Sequence Editor"
    }

    fn toolbar_buttons(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<Vec<Button>> {
        Some(toolbar_buttons(self, window, cx))
    }
}

impl Render for SequenceEditorPanel {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl IntoElement {
        if DemexUiState::selected_sequence(cx).read(cx).is_none() {
            return v_flex()
                .size_full()
                .justify_center()
                .items_center()
                .text_color(cx.theme().muted_foreground)
                .child("No sequence selected")
                .into_any_element();
        }

        let Some(sequence) = self.sequence.read(cx) else {
            return v_flex()
                .size_full()
                .justify_center()
                .items_center()
                .child("Loading")
                .into_any_element();
        };

        v_flex()
            .overflow_x_scrollbar()
            .w_full()
            .h_full()
            .child(
                h_flex().p_4().child(
                    Button::new("edit-sequence-name")
                        .on_click({
                            let sequence_id = sequence.sequence.id;
                            move |_, window, cx| {
                                WindowManager::open_edit_window::<SetPropertyWindow>(cx, |cx| {
                                    SetPropertyWindow::new(
                                        Object::Sequence(sequence_id),
                                        SequenceProperty::Name,
                                        SetPropertyWindowPropertyType::String,
                                        window,
                                        cx,
                                    )
                                });
                            }
                        })
                        .text()
                        .text_lg()
                        .font_bold()
                        .label(sequence.sequence.name.clone()),
                ),
            )
            .child(
                Table::new(&self.table_state)
                    .bordered(false)
                    .with_size(cx.ui_config().ui_size()),
            )
            .into_any_element()
    }
}
