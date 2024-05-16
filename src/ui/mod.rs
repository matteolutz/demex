#[allow(unused_imports)]
use crate::{
    dmx::output::{debug_dummy::DebugDummyOutput, dmx_serial::DMXSerialOutput},
    fixture::{handler::FixtureHandler, Fixture},
    lexer::Lexer,
    parser::Parser,
};
use crate::{
    fixture::{self, channel::FixtureChannel},
    parser::nodes::{action::Action, fixture_selector::FixtureSelector},
};

pub mod components;

pub struct UIApp {
    command_input: String,
    gm_slider_val: u8,
    fixture_handler: FixtureHandler,
    global_error: Option<Box<dyn std::error::Error>>,
    global_fixture_select: Option<FixtureSelector>,
}

fn get_test_fixture_handler(num_fixtures: u32) -> FixtureHandler {
    FixtureHandler::new(
        vec![
            Box::new(DebugDummyOutput::new(true)),
            /*Box::new(
            DMXSerialOutput::new("/dev/tty.usbserial-A10KPDBZ").expect("this shouldn't happen"),
            ),*/
        ],
        (1..num_fixtures + 1)
            .map(|id| {
                Fixture::new(
                    id,
                    format!("WASH {}", id),
                    vec![
                        FixtureChannel::position_pan_tilt(true),
                        FixtureChannel::intensity(true),
                        FixtureChannel::maintenance("Strobe"),
                        FixtureChannel::color_rgb(true),
                        FixtureChannel::maintenance("White"),
                        FixtureChannel::maintenance("WhiteFine"),
                        FixtureChannel::maintenance("ColorTemp"),
                        FixtureChannel::maintenance("ColorTint"),
                        FixtureChannel::maintenance("ColorMacro"),
                        FixtureChannel::maintenance("ColorMacroCrossfade"),
                        FixtureChannel::maintenance("Zoom"),
                        FixtureChannel::maintenance("ZoomFine"),
                        FixtureChannel::maintenance("PanTiltSpeed"),
                        FixtureChannel::maintenance("DeviceSettings"),
                    ],
                    1,
                    (id as u8 - 1) * 23 + 1,
                )
                .unwrap()
            })
            .collect(),
    )
    .expect("")
}

impl Default for UIApp {
    fn default() -> Self {
        let fh = get_test_fixture_handler(2);

        Self {
            command_input: String::new(),
            gm_slider_val: fh.grand_master(),
            fixture_handler: fh,
            global_error: None,
            global_fixture_select: None,
        }
    }
}

impl UIApp {
    pub fn run_cmd(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut l = Lexer::new(&self.command_input);
        let tokens = l.tokenize()?;

        let mut p = Parser::new(&tokens);
        let action = p.parse()?;

        match &action {
            Action::FixtureSelector(fixture_selector) => {
                self.global_fixture_select = Some(fixture_selector.clone());
            }
            _ => {}
        }

        let now = std::time::Instant::now();

        action.run(&mut self.fixture_handler)?;

        println!(
            "Execution of action {:?} took {:.2?}",
            action,
            now.elapsed()
        );

        Ok(())
    }
}

impl UIApp {
    fn update_fixture_control_window(&mut self, ctx: &eframe::egui::Context) {
        if self.global_fixture_select.is_none() {
            return;
        }

        let selected_fixtures = self.global_fixture_select.as_ref().unwrap().get_fixtures();

        if selected_fixtures.is_empty() {
            self.global_fixture_select = None;
            return;
        }

        let mut mutual_channel_types = self
            .fixture_handler
            .fixture(selected_fixtures[0])
            .unwrap()
            .channel_types()
            .clone();

        for fixture_id in selected_fixtures.iter().skip(1) {
            let fixture_channel_types = self
                .fixture_handler
                .fixture(*fixture_id)
                .unwrap()
                .channel_types();

            mutual_channel_types
                .retain(|channel_type| fixture_channel_types.contains(channel_type));
        }

        eframe::egui::Window::new("fixture control")
            .id(eframe::egui::Id::new("fixture_control_window"))
            .title_bar(true)
            .enabled(true)
            .collapsible(false)
            .scroll2([true; 2])
            .show(ctx, |ui| {
                ui.style_mut().spacing.item_spacing = [0.0, 20.0].into();

                ui.heading("fixture control");

                ui.horizontal(|ui| {
                    ui.style_mut().spacing.item_spacing = [20.0, 0.0].into();

                    for channel_type in mutual_channel_types {
                        match channel_type {
                            fixture::channel::FIXTURE_CHANNEL_INTENSITY_ID => {
                                ui.vertical(|ui| {
                                    ui.label("Intensity");
                                    ui.add(eframe::egui::Slider::from_get_set(
                                        0.0..=255.0,
                                        |val| {
                                            if let Some(val) = val {
                                                for fixture_id in selected_fixtures.iter() {
                                                    let intens = self
                                                        .fixture_handler
                                                        .fixture(*fixture_id)
                                                        .unwrap()
                                                        .intensity_ref()
                                                        .expect("");
                                                    *intens = Some(val as u8);
                                                }

                                                0.0
                                            } else {
                                                self.fixture_handler
                                                    .fixture(selected_fixtures[0])
                                                    .unwrap()
                                                    .intensity()
                                                    .expect("")
                                                    .unwrap_or(0)
                                                    as f64
                                            }
                                        },
                                    ));
                                });
                            }
                            fixture::channel::FIXTURE_CHANNEL_COLOR_RGB_ID => {
                                ui.vertical(|ui| {
                                    ui.label("Color RGB");

                                    let fixture_color = self
                                        .fixture_handler
                                        .fixture(selected_fixtures[0])
                                        .unwrap()
                                        .color_rgb()
                                        .expect("")
                                        .clone();

                                    if fixture_color.is_some() {
                                        let c = ui.color_edit_button_rgb(
                                            self.fixture_handler
                                                .fixture(selected_fixtures[0])
                                                .unwrap()
                                                .color_rgb_ref()
                                                .expect("")
                                                .as_mut()
                                                .unwrap(),
                                        );

                                        if c.changed() || c.clicked() {
                                            for fixture_id in selected_fixtures.iter().skip(1) {
                                                let color = self
                                                    .fixture_handler
                                                    .fixture(*fixture_id)
                                                    .unwrap()
                                                    .color_rgb_ref()
                                                    .expect("");

                                                *color = fixture_color.clone();
                                            }
                                        }
                                    } else {
                                        let button = ui.button("tag color");
                                        if button.clicked() {
                                            for fixture_id in selected_fixtures.iter() {
                                                let color = self
                                                    .fixture_handler
                                                    .fixture(*fixture_id)
                                                    .unwrap()
                                                    .color_rgb_ref()
                                                    .expect("");
                                                *color = Some([0.0; 3]);
                                            }
                                        }
                                    }
                                });
                            }
                            _ => {
                                ui.label("Not supported");
                            }
                        }

                        ui.separator();
                    }
                });

                let close_button = ui.button("Close");
                if close_button.clicked() {
                    self.global_fixture_select = None;
                }
            });
    }
}

impl eframe::App for UIApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        self.update_fixture_control_window(ctx);

        let _ = self.fixture_handler.update();

        eframe::egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
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
        });

        eframe::egui::CentralPanel::default().show(ctx, |ui| {
            let fixture_card_size = eframe::egui::vec2(75.0, 100.0);
            let window_width = ui.available_width();

            ui.with_layout(
                eframe::egui::Layout::top_down(eframe::egui::Align::LEFT),
                |ui| {
                    eframe::egui::ScrollArea::vertical()
                        .max_height(ui.available_height() - 60.0)
                        .show(ui, |ui| {
                            ui.heading("Fixtures");

                            let selectd_fixtures =
                                if let Some(fixture_select) = &self.global_fixture_select {
                                    fixture_select.get_fixtures()
                                } else {
                                    vec![]
                                };

                            for fixture_chunk in self
                                .fixture_handler
                                .fixtures()
                                .chunks((window_width / fixture_card_size.x) as usize - 1)
                            {
                                ui.horizontal(|ui| {
                                    for f in fixture_chunk {
                                        let fixture_intenstiy =
                                            f.intensity().expect("").unwrap_or(0);

                                        let (rect, _) = ui.allocate_exact_size(
                                            fixture_card_size,
                                            eframe::egui::Sense::click(),
                                        );

                                        ui.painter().rect_stroke(
                                            rect,
                                            10.0,
                                            eframe::egui::Stroke::new(
                                                2.0,
                                                if selectd_fixtures.contains(&f.id()) {
                                                    eframe::egui::Color32::from_rgb(0, 255, 0)
                                                } else {
                                                    eframe::egui::Color32::from_rgb(
                                                        255,
                                                        255,
                                                        255 - fixture_intenstiy,
                                                    )
                                                },
                                            ),
                                        );

                                        ui.put(rect, |ui: &mut eframe::egui::Ui| {
                                            ui.colored_label(
                                                eframe::egui::Color32::from_rgb(
                                                    255,
                                                    255,
                                                    255 - fixture_intenstiy,
                                                ),
                                                f.to_string(),
                                            )
                                        });
                                    }
                                });
                            }

                            ui.add_space(ui.available_height());
                        });
                },
            );

            eframe::egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
                ui.add_space(10.0);

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
            });

            ui.add_space(10.0);
        });
    }
}
