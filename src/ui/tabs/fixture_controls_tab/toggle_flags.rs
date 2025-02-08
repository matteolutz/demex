use crate::fixture::handler::FixtureHandler;

pub fn toggle_flags_controls_ui(
    ui: &mut eframe::egui::Ui,
    channel_name: &str,
    is_channel_home: bool,
    selected_fixtures: &[u32],
    fixture_handler: &mut FixtureHandler,
) {
    ui.vertical(|ui| {
        ui.set_width(100.0);
        ui.label(
            egui::RichText::from(channel_name).color(if is_channel_home {
                egui::Color32::PLACEHOLDER
            } else {
                egui::Color32::YELLOW
            }),
        );

        let unset_button = ui.button("Unset");
        if unset_button.clicked() {
            for fixture_id in selected_fixtures.iter() {
                fixture_handler
                    .fixture(*fixture_id)
                    .unwrap()
                    .unset_toggle_flags()
                    .expect("");
            }
        }

        ui.separator();

        let mut mutual_flags = fixture_handler
            .fixture(selected_fixtures[0])
            .unwrap()
            .toggle_flags();
        for f in selected_fixtures.iter().skip(1) {
            mutual_flags.retain(|flag| {
                fixture_handler
                    .fixture(*f)
                    .unwrap()
                    .toggle_flags()
                    .contains(flag)
            });
        }

        for flag in mutual_flags {
            let flag_button = ui.button(flag.clone());

            if flag_button.clicked() {
                for fixture_id in selected_fixtures.iter() {
                    fixture_handler
                        .fixture(*fixture_id)
                        .unwrap()
                        .set_toggle_flag(&flag)
                        .expect("");
                }
            }
        }
    });
}
