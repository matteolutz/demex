use gpui_component::table::{Column, TableDelegate};
use itertools::Itertools;

use crate::engine::state::DemexUiState;

pub struct FixtureListTable {
    columns: Vec<Column>,
}

impl Default for FixtureListTable {
    fn default() -> Self {
        Self {
            columns: vec![
                Column::new("id", "Id").width(60.0).ascending(),
                Column::new("patch", "Patch").width(60.0),
                Column::new("name", "Name").width(150.0),
                Column::new("dimmer", "Dimmer").width(100.0),
            ],
        }
    }
}

impl TableDelegate for FixtureListTable {
    fn columns_count(&self, _cx: &gpui::App) -> usize {
        self.columns.len()
    }

    fn rows_count(&self, cx: &gpui::App) -> usize {
        let patch = DemexUiState::patch(cx).read(cx);
        patch.fixtures().count()
    }

    fn column(&self, col_ix: usize, _cx: &gpui::App) -> &gpui_component::table::Column {
        &self.columns[col_ix]
    }

    fn render_td(
        &self,
        row_ix: usize,
        col_ix: usize,
        _window: &mut gpui::Window,
        cx: &mut gpui::App,
    ) -> impl gpui::IntoElement {
        let patch = DemexUiState::patch(cx).read(cx);

        let fixture = patch.fixtures().nth(row_ix).unwrap();
        let fixture_values = DemexUiState::fixture_values(cx)
            .read(cx)
            .get(&fixture.id())
            .unwrap();

        let column = &self.columns[col_ix];

        match column.key.as_ref() {
            "id" => fixture.id().to_string(),
            "patch" => format!("{}.{}", fixture.universe(), fixture.start_address()),
            "name" => fixture.name().to_string(),
            "dimmer" => {
                let dimmer_channels = fixture.channels_for_attribute(patch, "Dimmer").unwrap();
                let dimmer_values = dimmer_channels
                    .into_iter()
                    .map(|(dmx_channel, _, _)| {
                        fixture_values
                            .get(dmx_channel.name().as_ref())
                            .unwrap()
                            .to_opaque_string()
                    })
                    .join(",");

                dimmer_values
            }
            _ => unreachable!(),
        }
    }
}
