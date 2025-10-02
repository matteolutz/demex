use demex_ui::table::{Column, Table, TableDelegate};
use gpui::{App, Entity, Window, prelude::*, px};
use gpui::{Render, Styled, div};
use itertools::Itertools;

use crate::engine::DemexEngineHandler;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FixtureSheetColumnId {
    Id,
    Patch,
    Name,
}

pub struct FixtureSheet {
    table: Entity<Table<FixtureSheetTable>>,
}

impl FixtureSheet {
    pub fn new(window: &mut Window, cx: &mut App) -> Self {
        Self {
            table: cx.new(|cx| Table::new(FixtureSheetTable::default(), window, cx)),
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

pub struct FixtureSheetTable {
    columns: Vec<Column<FixtureSheetColumnId>>,
}

impl Default for FixtureSheetTable {
    fn default() -> Self {
        Self {
            columns: vec![
                Column::new(FixtureSheetColumnId::Id, "Id"),
                Column::new(FixtureSheetColumnId::Patch, "Patch"),
                Column::new(FixtureSheetColumnId::Name, "Name").with_width(px(200.0)),
            ],
        }
    }
}

impl TableDelegate for FixtureSheetTable {
    type RowId = u32;
    type ColId = FixtureSheetColumnId;

    fn column_count(&self, _cx: &gpui::App) -> usize {
        3
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
        }
    }
}
