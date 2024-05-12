use crate::{
    dmx::output::debug_dummy::DebugDummyOutput,
    fixture::{handler::FixtureHandler, Fixture, FixtureChannelType},
    lexer::Lexer,
    parser::Parser,
};

pub struct UIApp {
    command_input: String,
    fixture_handler: FixtureHandler,
}

impl Default for UIApp {
    fn default() -> Self {
        let mut fh = FixtureHandler::new();

        fh.add_output(Box::new(DebugDummyOutput {}));

        fh.add_fixture(Fixture::new(
            1,
            "PAR 1".to_string(),
            vec![FixtureChannelType::Intesity],
            1,
            1,
        ))
        .expect("This shouldn't happen :)");

        fh.add_fixture(Fixture::new(
            2,
            "PAR 2".to_string(),
            vec![FixtureChannelType::Intesity],
            2,
            1,
        ))
        .expect("This shouldn't happen :)");

        Self {
            command_input: String::new(),
            fixture_handler: fh,
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
            ui.heading("demex");
            ui.separator();

            ui.with_layout(
                eframe::egui::Layout::top_down(eframe::egui::Align::LEFT),
                |ui| {
                    eframe::egui::ScrollArea::vertical()
                        .max_height(ui.available_height() - 60.0)
                        .show(ui, |ui| {
                            ui.heading("Fixtures");
                            for fixture_chunk in self.fixture_handler.fixtures().chunks(5) {
                                ui.horizontal(|ui| {
                                    for f in fixture_chunk {
                                        /*let _ = ui.button(format!(
                                        "{} {} (U{}.A{})\n\n{}",
                                        f.id(),
                                        f.name(),
                                        f.universe(),
                                        f.start_address(),
                                        self.fixture_handler.fixture_state(f.id()).expect("")
                                        ));*/

                                        let fixture_state =
                                            self.fixture_handler.fixture_state(f.id()).expect("");
                                        let fixture_intenstiy = fixture_state.intensity();

                                        let (rect, _) = ui.allocate_exact_size(
                                            eframe::egui::vec2(75.0, 100.0),
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
                                                    "{}\n{} ({}.{})\n\n{}",
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
                        }

                        self.command_input.clear();
                        command_input_field.request_focus();
                    }
                },
            );
        });
    }
}
