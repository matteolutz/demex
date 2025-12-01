use gpui::{
    App, AppContext, Context, Entity, EventEmitter, FocusHandle, Focusable, IntoElement,
    ParentElement, Render, Styled, Subscription, Window, div,
};
use gpui_component::{
    Sizable,
    dock::{Panel, PanelEvent},
    table::{Table, TableState},
};

use crate::{
    engine::state::DemexUiState,
    ui2::{
        config::AppConfigExt,
        pane::panels::fixture_list::table::{FixtureListTable, FixtureListTableEntry},
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
        let patch = DemexUiState::patch(cx);
        let fixture_values = DemexUiState::fixture_values(cx);

        let delegate = {
            let patch = patch.read(cx);

            FixtureListTable::new(
                patch
                    .fixtures()
                    .map(|f| FixtureListTableEntry::from_patch(f, patch))
                    .collect(),
            )
        };

        let table_state = cx.new(|cx| TableState::new(delegate, window, cx).col_movable(false));

        let notify = |this: &mut FixtureListPanel, cx: &mut Context<Self>| {
            println!("rerendering");
            cx.notify();
            this.table_state.update(cx, |table, cx| table.refresh(cx));
        };

        let _subscriptions = vec![
            cx.observe(&patch, move |this, _, cx| notify(this, cx)),
            cx.observe(&fixture_values, move |this, _, cx| notify(this, cx)),
        ];

        Self {
            focus_handle: cx.focus_handle(),
            table_state,
            _subscriptions,
        }
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
        "fixture-list"
    }

    fn title(&self, _window: &gpui::Window, _cx: &App) -> gpui::AnyElement {
        "Fixture List".into_any_element()
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
