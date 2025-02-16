use crate::{
    fixture::{
        channel2::feature::{feature_type::FixtureFeatureType, feature_value::FixtureFeatureValue},
        handler::FixtureHandler,
        presets::PresetHandler,
    },
    utils::math::f32_to_coarse,
};

pub fn feature_f32_slider(
    ui: &mut egui::Ui,
    is_channel_home: bool,
    selected_fixtures: &[u32],
    feature_type: FixtureFeatureType,
    fixture_handler: &mut FixtureHandler,
    preset_handler: &PresetHandler,
    get_f32_val: impl FnOnce(FixtureFeatureValue) -> Option<f32>,
    make_feature_value: impl Fn(f32) -> FixtureFeatureValue,
) {
    ui.set_width(100.0);

    ui.vertical(|ui| {
        ui.label(
            egui::RichText::from(format!("{:?}", feature_type)).color(if is_channel_home {
                egui::Color32::PLACEHOLDER
            } else {
                egui::Color32::YELLOW
            }),
        );

        let value = fixture_handler
            .fixture_immut(selected_fixtures[0])
            .unwrap()
            .feature_value_programmer(feature_type, preset_handler)
            .unwrap();

        if let Some(value) = get_f32_val(value) {
            let mut slider_val = value;

            ui.add(
                egui::Slider::new(&mut slider_val, 0.0..=1.0)
                    .vertical()
                    .custom_formatter(|val, _| f32_to_coarse(val as f32).to_string()),
            );

            if slider_val != value {
                for fixture in selected_fixtures {
                    fixture_handler
                        .fixture(*fixture)
                        .unwrap()
                        .set_feature_value(make_feature_value(slider_val))
                        .unwrap();
                }
            }
        }
    });
}
