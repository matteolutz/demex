use crate::{
    fixture::{
        channel2::feature::{feature_type::FixtureFeatureType, feature_value::FixtureFeatureValue},
        handler::FixtureHandler,
        presets::PresetHandler,
        timing::TimingHandler,
    },
    ui::components::position_selector::PositionSelector,
};

pub fn position_pan_tilt_controls_ui(
    ui: &mut eframe::egui::Ui,
    is_channel_home: bool,
    selected_fixtures: &[u32],
    preset_handler: &PresetHandler,
    fixture_handler: &mut FixtureHandler,
    timing_handler: &TimingHandler,
) {
    ui.vertical(|ui| {
        ui.label(
            egui::RichText::from("Position Pan Tilt").color(if is_channel_home {
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
                        .set_feature_value(FixtureFeatureValue::PositionPanTilt {
                            pan: val.x,
                            tilt: val.y,
                            pan_tilt_speed: None,
                        })
                        .expect("");
                }

                Some(egui::vec2(0.0, 0.0))
            } else {
                if let FixtureFeatureValue::PositionPanTilt { pan, tilt, .. } = fixture_handler
                    .fixture(selected_fixtures[0])
                    .unwrap()
                    .feature_value_programmer(
                        FixtureFeatureType::PositionPanTilt,
                        preset_handler,
                        timing_handler,
                    )
                    .expect("")
                {
                    Some(egui::vec2(pan, tilt))
                } else {
                    None
                }
            }
        }));
    });
}
