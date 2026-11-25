use demex_core::command::parser::nodes::action::Action;
use gpui::{
    App, AppContext, Context, Entity, EventEmitter, FocusHandle, Focusable, IntoElement,
    ParentElement, Render, Styled, Subscription, Window, div,
};
use gpui_component::{
    Sizable,
    dock::PanelEvent,
    table::{DataTable, TableEvent, TableState},
};

use crate::{
    engine::{DemexEngineHandler, state::DemexUiState},
    ui2::{
        config::AppConfigExt,
        panels::{
            DemexPanel,
            fixture_list::table::{FixtureListTable, FixtureListTableEntry},
        },
    },
};

mod table;

pub struct FixtureListPanel {
    focus_handle: FocusHandle,

    table_state: Entity<TableState<FixtureListTable>>,

    _subscriptions: Vec<Subscription>,
}

impl FixtureListPanel {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let table_state = cx.new(|cx| {
            TableState::new(FixtureListTable::new(Self::get_table_data(cx)), window, cx)
                .col_movable(false)
        });

        let refresh_table = |this: &mut FixtureListPanel, cx: &mut Context<Self>| {
            this.table_state.update(cx, |table, cx| table.refresh(cx));
            cx.notify();
        };

        let _subscriptions = vec![
            cx.observe(&DemexUiState::patch(cx), move |this, _, cx| {
                this.table_state.update(cx, |table, cx| {
                    table.delegate_mut().update_data(Self::get_table_data(cx));
                    table.refresh(cx);
                });
                cx.notify();
            }),
            cx.observe(&DemexUiState::fixture_values(cx), move |this, _, cx| {
                refresh_table(this, cx)
            }),
            cx.observe(&DemexUiState::fixture_selection(cx), move |this, _, cx| {
                refresh_table(this, cx)
            }),
            cx.observe(&DemexUiState::highlight(cx), move |this, _, cx| {
                refresh_table(this, cx)
            }),
            cx.subscribe(&table_state, |_, state, evt, cx| match evt {
                TableEvent::DoubleClickedRow(row_ix) => {
                    let Some(fixture_path) = state.read(cx).delegate().row_fixture_path(*row_ix)
                    else {
                        return;
                    };
                    DemexEngineHandler::engine(cx)
                        .exec_ui(Action::AddFixturesToSelection(vec![*fixture_path]));
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

impl DemexPanel for FixtureListPanel {
    fn panel_type() -> super::DockWindowPanelType {
        super::DockWindowPanelType::FixtureList
    }

    fn deserialize(
        _dock_area: gpui::WeakEntity<gpui_component::dock::DockArea>,
        _panel_state: &gpui_component::dock::PanelState,
        _panel_info: &gpui_component::dock::PanelInfo,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        FixtureListPanel::new(window, cx)
    }
}

impl Render for FixtureListPanel {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl IntoElement {
        div().w_full().h_full().child(
            DataTable::new(&self.table_state)
                .bordered(false)
                .with_size(cx.ui_config().ui_size()),
        )
    }
}
