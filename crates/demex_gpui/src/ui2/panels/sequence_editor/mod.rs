use demex_core::{engine::comm::SequenceRequest, sequence::frontend::FrontendSequence};
use gpui::{
    App, AppContext, Context, Entity, EventEmitter, FocusHandle, Focusable, IntoElement,
    ParentElement, Render, Styled, Subscription, Window, div,
};
use gpui_component::{
    Sizable,
    button::Button,
    dock::{Panel, PanelEvent, register_panel},
    table::{Table, TableState},
    v_flex,
};

use crate::{
    engine::{DemexEngineHandler, state::DemexUiState},
    ui2::{
        config::AppConfigExt,
        panels::{sequence_editor::table::SequenceEditorTable, toolbar_buttons},
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

    sequence: Entity<Option<FrontendSequence>>,
    table_state: Entity<TableState<SequenceEditorTable>>,

    _subscriptions: Vec<Subscription>,
}

impl SequenceEditorPanel {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let table_state = cx.new(|cx| {
            TableState::new(SequenceEditorTable::new(None), window, cx).col_movable(false)
        });

        let sequence: Entity<Option<FrontendSequence>> = cx.new(|_| None);

        let _subscriptions = vec![
            cx.observe(&DemexUiState::selected_sequence(cx), |this, _, cx| {
                this.request_sequence(cx);
            }),
            cx.observe(&sequence, |this, sequence, cx| {
                let cues = sequence.read(cx).as_ref().map(|seq| seq.cues.clone());

                this.table_state.update(cx, |table, cx| {
                    table.delegate_mut().update_data(cues);
                    cx.notify();
                });
                cx.notify();
            }),
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
        let Some(sequence) = self.sequence.read(cx) else {
            return div().child("Loading");
        };

        v_flex()
            .w_full()
            .h_full()
            .child(sequence.name.clone())
            .child(
                Table::new(&self.table_state)
                    .bordered(false)
                    .with_size(cx.ui_config().ui_size()),
            )
    }
}
