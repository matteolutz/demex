use demex_core::event::DemexEvent;
use demex_ui::table::{Column, Table, TableDelegate};
use demex_ui::theme::ActiveTheme;
use gpui::{App, Entity, Window, prelude::*, px};
use gpui::{Render, Styled, div};
use itertools::Itertools;

use crate::engine::DemexEngineHandler;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum FixtureSheetColumnId {
    Id,
    Patch,
    Name,
    Dimmer,
}

pub struct FixtureSheet {
    table: Entity<Table<FixtureSheetTable>>,
}

impl FixtureSheet {
    pub fn new(window: &mut Window, cx: &mut App) -> Self {
        Self {
            table: cx.new(|cx| Table::new(FixtureSheetTable::new(window, cx), window, cx)),
        }
    }
}

impl Render for FixtureSheet {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        _cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        div().size_full().child(self.table.clone())
    }
}

#[derive(Clone)]
pub struct FixtureSheetTable {
    columns: Vec<Column<FixtureSheetColumnId>>,
}

impl FixtureSheetTable {
    pub fn new(window: &mut Window, cx: &mut Context<Table<Self>>) -> Self {
        let event_handler = DemexEngineHandler::event_handler(cx);
        cx.subscribe_in(
            &event_handler,
            window,
            |table, _, event, window, cx| match event {
                DemexEvent::FixtureSelectionChanged(_) | DemexEvent::FixtureValuesChanged(_) => {
                    table.refresh(window, cx);
                }
                _ => {}
            },
        )
        .detach();

        Self {
            columns: vec![
                Column::new(FixtureSheetColumnId::Id, "Id"),
                Column::new(FixtureSheetColumnId::Patch, "Patch"),
                Column::new(FixtureSheetColumnId::Name, "Name").with_width(px(200.0)),
                Column::new(FixtureSheetColumnId::Dimmer, "Dimmer"),
            ],
        }
    }

    fn get_attribute_value(
        &self,
        fixture_id: u32,
        attribute: &str,
        cx: &mut App,
    ) -> (bool, Option<String>) {
        let attribute_value =
            DemexEngineHandler::read_fixture_and_patch(cx, fixture_id, |f, patch| {
                f.get_attribute_value(patch.fixture_types(), attribute).ok()
            })
            .flatten();

        let is_active = attribute_value
            .as_ref()
            .is_some_and(|value| !value.is_home());

        let attribute_value_string = attribute_value.map(|value| {
            DemexEngineHandler::engine(cx)
                .preset_handler()
                .read(|ph| value.to_string(ph))
        });

        (is_active, attribute_value_string)
    }
}

impl TableDelegate for FixtureSheetTable {
    type RowId = u32;
    type ColId = FixtureSheetColumnId;

    fn column_count(&self, _cx: &gpui::App) -> usize {
        self.columns.len()
    }

    fn column(&self, col_ix: usize, _cx: &gpui::App) -> &demex_ui::table::Column<Self::ColId> {
        &self.columns[col_ix]
    }

    fn column_ix(&self, column_id: &Self::ColId, _cx: &gpui::App) -> usize {
        self.columns
            .iter()
            .position(|col| &col.id == column_id)
            .unwrap()
    }

    fn sorted_row_ids(&self, cx: &gpui::App) -> Vec<Self::RowId> {
        DemexEngineHandler::engine(cx).fixture_handler().read(|fh| {
            fh.fixtures()
                .iter()
                .map(|f| f.id())
                .sorted()
                .collect::<Vec<_>>()
        })
    }

    fn highlighted_row_ids(&self, cx: &App) -> Option<Vec<Self::RowId>> {
        DemexEngineHandler::engine(cx).state().read(|state| {
            state
                .fixture_selection
                .as_ref()
                .map(|selection| selection.fixtures().to_vec())
        })
    }

    fn render_cell(
        &self,
        row_id: &Self::RowId,
        col_ix: usize,
        _window: &mut gpui::Window,
        cx: &mut Context<demex_ui::table::Table<Self>>,
    ) -> impl IntoElement {
        let render_cell = |content| {
            div()
                .size_full()
                .flex()
                .items_center()
                .px_1()
                .child(content)
                .into_any_element()
        };

        let col_id = &self.column(col_ix, cx).id;
        match col_id {
            &FixtureSheetColumnId::Id => render_cell(
                DemexEngineHandler::read_fixture(cx, *row_id, |f| f.id())
                    .unwrap()
                    .to_string(),
            ),
            &FixtureSheetColumnId::Patch => render_cell(
                DemexEngineHandler::read_fixture(cx, *row_id, |f| {
                    format!("U{}.{}", f.universe(), f.start_address())
                })
                .unwrap()
                .to_string(),
            ),
            &FixtureSheetColumnId::Name => render_cell(
                DemexEngineHandler::read_fixture(cx, *row_id, |f| f.name().to_string())
                    .unwrap()
                    .to_string(),
            ),
            &FixtureSheetColumnId::Dimmer => {
                let (is_active, dimmer_value_string) =
                    self.get_attribute_value(*row_id, "Dimmer", cx);

                div()
                    .size_full()
                    .flex()
                    .items_center()
                    .px_1()
                    .when(is_active, |this| this.text_color(cx.theme().yellow))
                    .child(dimmer_value_string.unwrap_or_else(|| "N/A".to_string()))
                    .into_any_element()
            }
        }
    }
}
