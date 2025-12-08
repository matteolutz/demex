use demex_core::command::parser::nodes::action::Action;
use gpui::{
    App, AppContext, Context, Entity, EventEmitter, FocusHandle, Focusable, IntoElement,
    ParentElement, Render, Styled, Subscription, Window, div,
};
use gpui_component::{
    Sizable,
    button::Button,
    dock::{Panel, PanelEvent, register_panel},
    table::{Table, TableEvent, TableState},
};

use crate::{
    engine::{DemexEngineHandler, state::DemexUiState},
    ui2::{
        config::AppConfigExt,
        panels::{
            fixture_list::table::{FixtureListTable, FixtureListTableEntry},
            toolbar_buttons,
        },
    },
};

mod table;

const FIXTURE_LIST_PANEL_NAME: &str = "demex-fixture-list";

pub(super) fn register(cx: &mut App) {
    register_panel(cx, FIXTURE_LIST_PANEL_NAME, |_, _, _, window, cx| {
        Box::new(cx.new(|cx| FixtureListPanel::new(window, cx)))
    });
}

pub struct FixtureListPanel {
    focus_handle: FocusHandle,

    table_state: Entity<TableState<FixtureListTable>>,

    _subscriptions: Vec<Subscription>,
}

impl FixtureListPanel {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let patch = DemexUiState::patch(cx);
        let fixture_values = DemexUiState::fixture_values(cx);
        let fixture_selection = DemexUiState::fixture_selection(cx);

        let table_state = cx.new(|cx| {
            TableState::new(FixtureListTable::new(Self::get_table_data(cx)), window, cx)
                .col_movable(false)
        });

        let refresh_table = |this: &mut FixtureListPanel, cx: &mut Context<Self>| {
            this.table_state.update(cx, |table, cx| table.refresh(cx));
            cx.notify();
        };

        let _subscriptions = vec![
            cx.observe(&patch, move |this, _, cx| {
                this.table_state.update(cx, |table, cx| {
                    table.delegate_mut().update_data(Self::get_table_data(cx));
                    table.refresh(cx);
                });
                cx.notify();
            }),
            cx.observe(&fixture_values, move |this, _, cx| refresh_table(this, cx)),
            cx.observe(&fixture_selection, move |this, _, cx| {
                refresh_table(this, cx)
            }),
            cx.subscribe(&table_state, |_, state, evt, cx| match evt {
                TableEvent::DoubleClickedRow(row_ix) => {
                    let Some(fixture_id) = state.read(cx).delegate().row_fixture_id(*row_ix) else {
                        return;
                    };
                    DemexEngineHandler::engine(cx)
                        .exec_ui(Action::AddFixturesToSelection(vec![fixture_id]));
                }
                _ => {}
            }),
        ];

        Self {
            focus_handle: cx.focus_handle(),
            table_state,
            _subscriptions,
        }
    }

    fn get_table_data(cx: &App) -> Vec<FixtureListTableEntry> {
        let patch = DemexUiState::patch(cx).read(cx);
        patch
            .fixtures()
            .map(|f| FixtureListTableEntry::from_patch_and_selection(f, patch))
            .collect()
    }
}

impl EventEmitter<PanelEvent> for FixtureListPanel {}
impl Focusable for FixtureListPanel {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Panel for FixtureListPanel {
    fn panel_name(&self) -> &'static str {
        FIXTURE_LIST_PANEL_NAME
    }

    fn title(&mut self, _window: &mut gpui::Window, _cx: &mut Context<Self>) -> impl IntoElement {
        "Fixture List"
    }

    fn toolbar_buttons(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<Vec<Button>> {
        Some(toolbar_buttons(self, window, cx))
    }
}

impl Render for FixtureListPanel {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl IntoElement {
        div().w_full().h_full().child(
            Table::new(&self.table_state)
                .bordered(false)
                .with_size(cx.ui_config().ui_size()),
        )
    }
}
