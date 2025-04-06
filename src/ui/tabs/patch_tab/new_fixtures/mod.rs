use crate::ui::context::DemexUiContext;

#[derive(Default, Clone)]
struct State {
    pub fixture_type: Option<String>,
    pub fixture_mode: Option<u32>,

    pub universe: u16,

    pub start_address: u8,
    pub num_fixutres: u8,

    pub name_format: String,
}

pub struct PatchNewFixturesComponent<'a> {
    context: &'a mut DemexUiContext,
    id_source: egui::Id,
}

impl<'a> PatchNewFixturesComponent<'a> {
    pub fn new(context: &'a mut DemexUiContext) -> Self {
        Self {
            context,
            id_source: egui::Id::new("DemexPatchNewFixturesComponent"),
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        let mut state = ui
            .data(|data| data.get_temp::<State>(self.id_source))
            .unwrap_or_default();

        let fixture_handler = self.context.fixture_handler.read();
        let patch = fixture_handler.patch();

        let fixture_types = patch.fixture_types();

        egui::ComboBox::new(
            self.id_source.with("FixtureTypeSelection"),
            "Select Fixture Type",
        )
        .selected_text(
            state
                .fixture_type
                .as_ref()
                .map(|fixture_type| fixture_types.get(fixture_type).unwrap().name.clone())
                .unwrap_or("None".to_owned()),
        )
        .show_ui(ui, |ui| {
            ui.selectable_value(&mut state.fixture_type, None, "None");

            for (fixture_type_id, fixture_type) in fixture_types {
                ui.selectable_value(
                    &mut state.fixture_type,
                    Some(fixture_type_id.clone()),
                    &fixture_type.name,
                );
            }
        });

        if let Some(fixture_type) = &state.fixture_type {
            let fixture_modes = &fixture_types.get(fixture_type).unwrap().modes;

            let fixture_mode = state.fixture_mode.and_then(|mode| fixture_modes.get(&mode));

            egui::ComboBox::new(
                self.id_source.with("FixtureModeSelection"),
                "Select Fixture Mode",
            )
            .selected_text(fixture_mode.map_or("None".to_owned(), |m| m.name.clone()))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut state.fixture_mode, None, "None");

                for (mode_id, mode) in fixture_modes {
                    ui.selectable_value(
                        &mut state.fixture_mode,
                        Some(*mode_id),
                        format!("{} ({} Channels)", mode.name, mode.channel_types.len()),
                    );
                }
            });
        }

        egui_probe::Probe::new(&mut state.universe)
            .with_header("Universe")
            .show(ui);

        egui_probe::Probe::new(&mut state.start_address)
            .with_header("Start Address")
            .show(ui);

        egui_probe::Probe::new(&mut state.num_fixutres)
            .with_header("Number of Fixtures")
            .show(ui);

        egui::TextEdit::singleline(&mut state.name_format)
            .hint_text("Name Format")
            .show(ui);

        ui.data_mut(|data| data.insert_temp(self.id_source, state));
    }
}
