use demex_ui::table::{Column, Table, TableDelegate};
use gpui::{App, Entity, Timer, Window, px};
use gpui::{div, prelude::*};
use itertools::Itertools;

use crate::engine::DemexEngineHandler;

const PERFORMANCE_UPDATE_INTERVAL: std::time::Duration = std::time::Duration::from_secs(1);

pub struct Performance {
    table: Entity<Table<PerformanceTable>>,
}

impl Performance {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let s = Self {
            table: cx.new(|cx| Table::new(PerformanceTable::default(), window, cx)),
        };

        s.update(cx);

        s
    }

    fn update(&self, cx: &mut Context<Self>) {
        cx.notify();

        cx.spawn(async move |this, cx| {
            Timer::after(PERFORMANCE_UPDATE_INTERVAL).await;
            this.update(cx, |this, cx| this.update(cx)).unwrap();
        })
        .detach();
    }
}

impl Render for Performance {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        _cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        div().size_full().child(self.table.clone())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, strum_macros::Display)]
pub enum PerformanceTableColumnId {
    Name,
    ThreadId,
    Dt,
    Fps,
    MaxDt,
    MinFps,
}

pub struct PerformanceTable {
    columns: Vec<Column<PerformanceTableColumnId>>,
}

impl Default for PerformanceTable {
    fn default() -> Self {
        Self {
            columns: vec![
                Column::new_auto(PerformanceTableColumnId::Name).with_width(px(200.0)),
                Column::new_auto(PerformanceTableColumnId::ThreadId),
                Column::new_auto(PerformanceTableColumnId::Dt),
                Column::new_auto(PerformanceTableColumnId::Fps),
                Column::new_auto(PerformanceTableColumnId::MaxDt),
                Column::new_auto(PerformanceTableColumnId::MinFps),
            ],
        }
    }
}

impl TableDelegate for PerformanceTable {
    type RowId = String;

    type ColId = PerformanceTableColumnId;

    fn column_count(&self, _cx: &App) -> usize {
        self.columns.len()
    }

    fn column(&self, col_ix: usize, _cx: &App) -> &Column<Self::ColId> {
        &self.columns[col_ix]
    }

    fn column_ix(&self, column_id: &Self::ColId, _cx: &App) -> usize {
        self.columns
            .iter()
            .position(|col| &col.id == column_id)
            .unwrap()
    }

    fn sorted_row_ids(&self, cx: &App) -> Vec<Self::RowId> {
        DemexEngineHandler::engine(cx)
            .stats()
            .read(|stats| stats.stats().keys().cloned().sorted().collect::<Vec<_>>())
    }

    fn render_cell(
        &self,
        row_id: &Self::RowId,
        col_ix: usize,
        _window: &mut Window,
        cx: &mut Context<Table<Self>>,
    ) -> impl IntoElement {
        let (stats, thread_id) = DemexEngineHandler::engine(cx).stats().read(|stats| {
            (
                stats.stats().get(row_id).cloned().unwrap(),
                stats.thread_id(row_id).unwrap(),
            )
        });

        let col = self.column(col_ix, cx);

        div()
            .size_full()
            .flex()
            .items_center()
            .px_1()
            .child(match col.id {
                PerformanceTableColumnId::Name => row_id.clone(),
                PerformanceTableColumnId::ThreadId => format!("{:?}", thread_id),
                PerformanceTableColumnId::Dt => format!("{:.2}", stats.dt()),
                PerformanceTableColumnId::Fps => format!("{:.2}", 1.0 / stats.dt()),
                PerformanceTableColumnId::MaxDt => format!("{:.2}", stats.max_dt()),
                PerformanceTableColumnId::MinFps => format!("{:.2}", 1.0 / stats.max_dt()),
            })
            .into_any_element()
    }
}
