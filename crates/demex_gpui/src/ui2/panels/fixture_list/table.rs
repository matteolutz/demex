use demex_core::{fixture::GdtfFixturePatch, patch::Patch};
use gpui::{IntoElement, ParentElement, Styled, div, prelude::FluentBuilder};
use gpui_component::{
    ActiveTheme,
    table::{Column, ColumnSort, TableDelegate},
};
use itertools::Itertools;

use crate::engine::state::DemexUiState;

pub struct FixtureListTableEntry {
    pub id: u32,
    pub patch: (u16, u16),
    pub fixture_type_name: String,
}

impl FixtureListTableEntry {
    pub fn from_patch_and_selection(value: &GdtfFixturePatch, patch: &Patch) -> Self {
        let fixture_type_name = patch
            .fixture_type(value.fixture_type_id)
            .map(|ft| {
                ft.name
                    .as_ref()
                    // Try default "name" attributes first..
                    .map(|name| name.as_ref().to_string())
                    // ..if not present, use "short_name"
                    .unwrap_or_else(|| ft.short_name.clone())
            })
            .unwrap_or_else(|| "(unknown)".into());

        Self {
            id: value.id,
            patch: (value.universe, value.start_address),
            fixture_type_name: fixture_type_name,
        }
    }
}

pub struct FixtureListTable {
    data: Vec<FixtureListTableEntry>,
    columns: Vec<Column>,
}

impl FixtureListTable {
    pub fn new(data: Vec<FixtureListTableEntry>) -> Self {
        let mut s = Self {
            data,
            columns: vec![
                Column::new("id", "Id").width(60.0).ascending(),
                Column::new("patch", "Patch").width(60.0).sortable(),
                Column::new("name", "Name").width(150.0),
                Column::new("fixture_type", "Fixture Type")
                    .width(150.0)
                    .sortable(),
                Column::new("dimmer", "Dimmer").width(100.0),
            ],
        };

        s.sort();
        s
    }

    pub fn update_data(&mut self, data: Vec<FixtureListTableEntry>) {
        self.data = data;
        self.sort();
    }

    pub fn sort(&mut self) {
        self._perform_sort(0, ColumnSort::Ascending);
    }

    fn _perform_sort(&mut self, col_ix: usize, sort: gpui_component::table::ColumnSort) {
        let col = &self.columns[col_ix];
        match col.key.as_ref() {
            "id" => match sort {
                ColumnSort::Ascending => self.data.sort_by(|a, b| a.id.cmp(&b.id)),
                ColumnSort::Descending => self.data.sort_by(|a, b| b.id.cmp(&a.id)),
                ColumnSort::Default => {}
            },
            "patch" => match sort {
                ColumnSort::Ascending => self.data.sort_by(|a, b| a.patch.cmp(&b.patch)),
                ColumnSort::Descending => self.data.sort_by(|a, b| b.patch.cmp(&a.patch)),
                ColumnSort::Default => {}
            },

            "fixture_type" => match sort {
                ColumnSort::Ascending => self
                    .data
                    .sort_by(|a, b| a.fixture_type_name.cmp(&b.fixture_type_name)),
                ColumnSort::Descending => self
                    .data
                    .sort_by(|a, b| b.fixture_type_name.cmp(&a.fixture_type_name)),
                ColumnSort::Default => {}
            },
            _ => unreachable!(),
        }
    }
}

impl TableDelegate for FixtureListTable {
    fn columns_count(&self, _cx: &gpui::App) -> usize {
        self.columns.len()
    }

    fn rows_count(&self, _cx: &gpui::App) -> usize {
        self.data.len()
    }

    fn column(&self, col_ix: usize, _cx: &gpui::App) -> &gpui_component::table::Column {
        &self.columns[col_ix]
    }

    fn perform_sort(
        &mut self,
        col_ix: usize,
        sort: gpui_component::table::ColumnSort,
        _window: &mut gpui::Window,
        _cx: &mut gpui::Context<gpui_component::table::TableState<Self>>,
    ) {
        self._perform_sort(col_ix, sort);
    }

    fn render_td(
        &self,
        row_ix: usize,
        col_ix: usize,
        _window: &mut gpui::Window,
        cx: &mut gpui::App,
    ) -> impl gpui::IntoElement {
        let column = &self.columns[col_ix];
        let entry = &self.data[row_ix];

        let patch = DemexUiState::patch(cx).read(cx);

        let fixture_selection = DemexUiState::fixture_selection(cx).read(cx).as_ref();
        let is_selected = fixture_selection.is_some_and(|s| s.has_fixture(entry.id));

        let fixture = patch.fixture(entry.id).unwrap();

        let fixture_values = DemexUiState::fixture_values(cx)
            .read(cx)
            .get(&fixture.id())
            .unwrap();

        match column.key.as_ref() {
            "id" => fixture.id().to_string().into_any_element(),
            "patch" => {
                format!("{}.{}", fixture.universe(), fixture.start_address()).into_any_element()
            }
            "name" => div()
                .when(is_selected, |div| div.text_color(cx.theme().green))
                .child(fixture.name().to_string())
                .into_any_element(),
            "fixture_type" => entry.fixture_type_name.clone().into_any_element(),
            "dimmer" => {
                let dimmer_channels = fixture.channels_for_attribute(patch, "Dimmer").unwrap();
                let is_home = dimmer_channels.iter().all(|(dmx_channel, _, _)| {
                    fixture_values
                        .get(dmx_channel.name().as_ref())
                        .unwrap()
                        .is_home()
                });
                let dimmer_values = dimmer_channels
                    .into_iter()
                    .map(|(dmx_channel, _, _)| {
                        fixture_values
                            .get(dmx_channel.name().as_ref())
                            .unwrap()
                            .to_opaque_string()
                    })
                    .join(",");

                div()
                    .when(!is_home, |div| div.text_color(cx.theme().yellow))
                    .child(dimmer_values)
                    .into_any_element()
            }
            _ => unreachable!(),
        }
    }
}
