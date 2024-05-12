use crate::{
    dmx::output::debug_dummy::DebugDummyOutput,
    fixture::{handler::FixtureHandler, Fixture, FixtureChannelType},
    lexer::Lexer,
    parser::Parser,
};

pub struct UIApp {
    command_input: String,
    gm_slider_val: u8,
    fixture_handler: FixtureHandler,
    global_error: Option<Box<dyn std::error::Error>>,
}

fn get_test_fixture_handler(num_fixtures: u32) -> FixtureHandler {
    let mut fh = FixtureHandler::new();

    fh.add_output(Box::new(DebugDummyOutput {}));

    for i in 1..num_fixtures + 1 {
        fh.add_fixture(Fixture::new(
            i,
            format!("PAR {}", i),
            vec![FixtureChannelType::Intesity],
            1,
            i as u8,
        ))
        .expect("This shouldn't happen :)");
    }

    fh
}

impl Default for UIApp {
    fn default() -> Self {
        let fh = get_test_fixture_handler(10);

        Self {
            command_input: String::new(),
            gm_slider_val: fh.grand_master(),
            fixture_handler: fh,
            global_error: None,
        }
    }
}

impl UIApp {
    pub fn run_cmd(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut l = Lexer::new(&self.command_input);
        let tokens = l.tokenize()?;

        let mut p = Parser::new(&tokens);
        let action = p.parse()?;

        action.run(&mut self.fixture_handler)?;

        Ok(())
    }
}

impl eframe::App for UIApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        let _ = self.fixture_handler.update();

        eframe::egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("demex");
                ui.separator();
                let slider =
                    ui.add(eframe::egui::Slider::new(&mut self.gm_slider_val, 0..=255).text("GM"));

                if slider.changed() {
                    self.fixture_handler.set_grand_master(self.gm_slider_val);
                }
            });

            if let Some(error) = &self.global_error {
                ui.colored_label(eframe::egui::Color32::RED, format!("Error: {}", error));
            }

            ui.separator();

            let fixture_card_size = eframe::egui::vec2(75.0, 100.0);
            let window_width = ui.available_width();

            ui.with_layout(
                eframe::egui::Layout::top_down(eframe::egui::Align::LEFT),
                |ui| {
                    eframe::egui::ScrollArea::vertical()
                        .max_height(ui.available_height() - 60.0)
                        .show(ui, |ui| {
                            ui.heading("Fixtures");

                            for fixture_chunk in self
                                .fixture_handler
                                .fixtures()
                                .chunks((window_width / fixture_card_size.x) as usize - 1)
                            {
                                ui.horizontal(|ui| {
                                    for f in fixture_chunk {
                                        let fixture_state =
                                            self.fixture_handler.fixture_state(f.id()).expect("");
                                        let fixture_intenstiy = fixture_state.intensity();

                                        let (rect, _) = ui.allocate_exact_size(
                                            fixture_card_size,
                                            eframe::egui::Sense::click(),
                                        );

                                        ui.painter().rect_stroke(
                                            rect,
                                            10.0,
                                            eframe::egui::Stroke::new(
                                                2.0,
                                                eframe::egui::Color32::from_rgba_unmultiplied(
                                                    255,
                                                    255,
                                                    255 - fixture_intenstiy,
                                                    255,
                                                ),
                                            ),
                                        );

                                        ui.put(rect, |ui: &mut eframe::egui::Ui| {
                                            ui.colored_label(
                                                eframe::egui::Color32::from_rgba_unmultiplied(
                                                    255,
                                                    255,
                                                    255 - fixture_intenstiy,
                                                    255,
                                                ),
                                                format!(
                                                    "{}\n{} (U{}.{})\n\n{}",
                                                    f.name(),
                                                    f.id(),
                                                    f.universe(),
                                                    f.start_address(),
                                                    fixture_state
                                                ),
                                            )
                                        });
                                    }
                                });
                            }

                            ui.add_space(ui.available_height());
                        });

                    ui.separator();

                    let command_label = ui.label("Command");
                    let command_input_field = ui
                        .add_sized(
                            ui.available_size(),
                            eframe::egui::TextEdit::singleline(&mut self.command_input)
                                .font(eframe::egui::FontId::new(
                                    16.0,
                                    eframe::egui::FontFamily::Monospace,
                                ))
                                .text_color(eframe::egui::Color32::YELLOW)
                                .vertical_align(eframe::egui::Align::Center),
                        )
                        .labelled_by(command_label.id);

                    if command_input_field.lost_focus()
                        && command_input_field
                            .ctx
                            .input(|i| i.key_pressed(eframe::egui::Key::Enter))
                    {
                        if let Err(e) = self.run_cmd() {
                            eprintln!("{}", e);
                            self.global_error = Some(e);
                        }

                        self.command_input.clear();
                        command_input_field.request_focus();
                    }
                },
            );
        });
    }
}
