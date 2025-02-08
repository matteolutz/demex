use crate::{
    fixture::{
        channel::{
            value::{FixtureChannelDiscreteValue, FixtureChannelValue, FixtureChannelValueTrait},
            FIXTURE_CHANNEL_POSITION_PAN_TILT_ID,
        },
        handler::FixtureHandler,
        presets::PresetHandler,
    },
    ui::components::position_selector::PositionSelector,
};

pub fn position_controls_ui(
    ui: &mut eframe::egui::Ui,
    channel_name: &str,
    is_channel_home: bool,
    selected_fixtures: &[u32],
    preset_handler: &PresetHandler,
    fixture_handler: &mut FixtureHandler,
) {
    ui.vertical(|ui| {
        ui.label(
            egui::RichText::from(channel_name).color(if is_channel_home {
                egui::Color32::PLACEHOLDER
            } else {
                egui::Color32::YELLOW
            }),
        );

        ui.add(PositionSelector::new(|val| {
            if let Some(val) = val {
                for fixture_id in selected_fixtures.iter() {
                    fixture_handler
                        .fixture(*fixture_id)
                        .unwrap()
                        .set_position_pan_tilt(FixtureChannelValue::Discrete(
                            FixtureChannelDiscreteValue::Pair(val.into()),
                        ))
                        .expect("");
                }

                Some(eframe::egui::vec2(0.0, 0.0))
            } else {
                let pos_val = fixture_handler
                    .fixture(selected_fixtures[0])
                    .unwrap()
                    .position_pan_tilt_programmer()
                    .expect("");

                let pos = pos_val
                    // TODO: change this, to use the corresponding fixture
                    .as_pair(
                        preset_handler,
                        selected_fixtures[0],
                        FIXTURE_CHANNEL_POSITION_PAN_TILT_ID,
                    )
                    .expect("");

                Some(eframe::egui::vec2(pos[0], pos[1]))
            }
        }));
    });
}
