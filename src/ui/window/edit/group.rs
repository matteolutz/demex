use egui_probe::Probe;

use crate::{
    fixture::{handler::FixtureHandler, presets::group::FixtureGroup},
    ui::components::{
        fixture_selection_editor::fixture_selection_editor, separator::padded_separator,
    },
};

pub fn edit_group_ui(
    ui: &mut egui::Ui,
    group: &mut FixtureGroup,
    fixture_handler: &FixtureHandler,
) {
    ui.heading("Group information");

    Probe::new(group.name_mut()).with_header("Name").show(ui);

    padded_separator(ui);

    ui.heading("Fixture selection");

    fixture_selection_editor(ui, group.fixture_selection_mut(), fixture_handler);
}
