use itertools::Itertools;

pub fn ui(ui: &mut eframe::egui::Ui, context: &mut super::DemexUiContext) {
    ui.horizontal(|ui| {
        for id in context.preset_handler.fader_ids().iter().sorted() {
            ui.vertical(|ui| {
                ui.set_min_width(100.0);

                ui.label(context.preset_handler.fader(*id).unwrap().name());
                ui.label(
                    egui::RichText::from(format!(
                        "{}",
                        context.preset_handler.fader(*id).unwrap().config()
                    ))
                    .small(),
                );

                ui.add(
                    eframe::egui::Slider::from_get_set(0.0..=1.0, |val| {
                        if let Some(val) = val {
                            // TODO: this is ugly
                            let fader = context.preset_handler.fader(*id).unwrap().clone();
                            fader.activate(
                                &mut context.fixture_handler,
                                &mut context.preset_handler,
                            );

                            context
                                .preset_handler
                                .fader_mut(*id)
                                .unwrap()
                                .set_value(val as f32);

                            val
                        } else {
                            context.preset_handler.fader(*id).unwrap().value() as f64
                        }
                    })
                    .orientation(egui::SliderOrientation::Vertical),
                );
            });
        }
    });
}
