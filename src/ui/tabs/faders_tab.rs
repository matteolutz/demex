use itertools::Itertools;

use crate::lexer::token::Token;

pub fn ui(ui: &mut eframe::egui::Ui, context: &mut super::DemexUiContext) {
    let mut fixture_handler = context.fixture_handler.write();
    let preset_handler = context.preset_handler.read();
    let mut updatable_handler = context.updatable_handler.write();

    ui.horizontal(|ui| {
        for id in updatable_handler.executor_ids().iter().sorted() {
            ui.vertical(|ui| {
                ui.set_min_width(100.0);

                ui.label(
                    egui::RichText::from(
                        updatable_handler
                            .executor(*id)
                            .unwrap()
                            .display_name(&preset_handler),
                    )
                    .color(
                        if updatable_handler.executor(*id).unwrap().is_active() {
                            ecolor::Color32::YELLOW
                        } else {
                            ecolor::Color32::PLACEHOLDER
                        },
                    ),
                );
                ui.label(
                    egui::RichText::from(format!(
                        "{}",
                        updatable_handler
                            .executor(*id)
                            .unwrap()
                            .display_name(&preset_handler)
                    ))
                    .small(),
                );

                ui.add(
                    eframe::egui::Slider::from_get_set(0.0..=1.0, |val| {
                        if let Some(val) = val {
                            // TODO: this is ugly

                            let fader = updatable_handler.executor_mut(*id).unwrap();

                            fader.set_value(val as f32, &mut fixture_handler, &preset_handler, 0.0);

                            val
                        } else {
                            updatable_handler.executor(*id).unwrap().value() as f64
                        }
                    })
                    .vertical(),
                );

                if ui.button("Home").clicked() {
                    updatable_handler
                        .executor_mut(*id)
                        .unwrap()
                        .stop(&mut fixture_handler, &preset_handler);
                }

                if ui.button("Sel").clicked() {
                    context
                        .command
                        .extend_from_slice(&[Token::KeywordExecutor, Token::Integer(*id)]);
                }
            });
        }

        ui.add_space(25.0);
        ui.separator();

        // Grand master
        ui.vertical(|ui| {
            ui.set_min_width(100.0);
            ui.label(egui::RichText::from("Grandmaster").color(ecolor::Color32::LIGHT_RED));
            ui.add(
                eframe::egui::Slider::new(fixture_handler.grand_master_mut(), 0..=255).vertical(),
            );
        });
    });
}
