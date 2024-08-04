use itertools::Itertools;
use std::collections::HashMap;
use tabs::DemexTabs;

#[allow(unused_imports)]
use crate::{
    dmx::output::{debug_dummy::DebugDummyOutput, dmx_serial::DMXSerialOutput},
    fixture::{handler::FixtureHandler, Fixture},
    lexer::Lexer,
    parser::Parser,
};
use crate::{
    fixture::{
        channel::FixtureChannel,
        presets::PresetHandler,
        sequence::{cue::Cue, runtime::SequenceRuntime, Sequence},
    },
    lexer::token::Token,
    parser::nodes::{
        action::Action,
        fixture_selector::{AtomicFixtureSelector, FixtureSelector},
    },
};

pub mod components;
pub mod tabs;

pub struct DemexUiContext {
    fixture_handler: FixtureHandler,
    preset_handler: PresetHandler,
    global_fixture_select: Option<FixtureSelector>,
    command: Vec<Token>,
    sequence_runtimes: Vec<SequenceRuntime>,
}

pub struct DemexUiApp {
    command_input: String,
    is_command_input_empty: bool,
    gm_slider_val: u8,
    context: DemexUiContext,
    global_error: Option<Box<dyn std::error::Error>>,
    max_update_time: Option<std::time::Duration>,
    tabs: DemexTabs,
}

fn get_test_fixture_handler() -> FixtureHandler {
    let mut fixtures = Vec::new();
    fixtures.extend((1..=2).map(|id| {
        Fixture::new(
            id,
            format!("WASH {}", id),
            vec![
                FixtureChannel::position_pan_tilt(true),
                FixtureChannel::intensity(true),
                FixtureChannel::strobe(),
                FixtureChannel::color_rgb(true),
                FixtureChannel::maintenance("White"),
                FixtureChannel::maintenance("WhiteFine"),
                FixtureChannel::maintenance("ColorTemp"),
                FixtureChannel::maintenance("ColorTint"),
                FixtureChannel::maintenance("ColorMacro"),
                FixtureChannel::maintenance("ColorMacroCrossfade"),
                FixtureChannel::zoom(true),
                FixtureChannel::maintenance("PanTiltSpeed"),
                FixtureChannel::toggle_flags(HashMap::from([
                    ("Turn On".to_owned(), 131u8),
                    ("Turn Off".to_owned(), 231u8),
                ])),
            ],
            1,
            (id as u16 - 1) * 40 + 411,
        )
        .unwrap()
    }));

    for i in 0..8 {
        fixtures.push(
            Fixture::new(
                i + 3,
                format!("PAR {}", i + 2),
                vec![FixtureChannel::intensity(false)],
                1,
                8 - i as u16,
            )
            .unwrap(),
        )
    }

    FixtureHandler::new(
        vec![
            Box::new(DebugDummyOutput::new(true)),
            /*Box::new(
            DMXSerialOutput::new("/dev/tty.usbserial-A10KPDBZ").expect("this shouldn't happen"),
            )*/
        ],
        fixtures,
    )
    .expect("")
}

fn get_test_preset_handler() -> PresetHandler {
    let mut ph = PresetHandler::new();

    // Groups
    ph.record_group(
        FixtureSelector::Atomic(AtomicFixtureSelector::FixtureRange(1, 2)),
        1,
    )
    .expect("");
    ph.rename_group(1, "Washes".to_owned()).expect("");

    ph.record_group(
        FixtureSelector::Atomic(AtomicFixtureSelector::FixtureRange(3, 10)),
        2,
    )
    .expect("");
    ph.rename_group(2, "PARs".to_owned()).expect("");

    // Sequences
    let mut seq = Sequence::new(1);
    seq.add_cue(Cue::new(HashMap::from([(
        1,
        vec![FixtureChannel::Intensity(true, 1.0)],
    )])));
    seq.add_cue(Cue::new(HashMap::from([(
        2,
        vec![FixtureChannel::Intensity(true, 1.0)],
    )])));
    seq.add_cue(Cue::new(HashMap::from([(
        1,
        vec![FixtureChannel::Intensity(true, 0.0)],
    )])));
    seq.add_cue(Cue::new(HashMap::from([(
        2,
        vec![FixtureChannel::Intensity(true, 0.0)],
    )])));

    ph.add_sequence(seq);

    ph
}

impl Default for DemexUiApp {
    fn default() -> Self {
        let fh = get_test_fixture_handler();
        let ph = get_test_preset_handler();

        Self {
            command_input: String::new(),
            is_command_input_empty: true,
            gm_slider_val: fh.grand_master(),
            context: DemexUiContext {
                fixture_handler: fh,
                preset_handler: ph,
                global_fixture_select: None,
                command: Vec::new(),
                sequence_runtimes: Vec::new(),
            },
            global_error: None,
            max_update_time: None,
            tabs: DemexTabs::new(),
        }
    }
}

impl DemexUiApp {
    pub fn run_cmd(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        /*let mut l = Lexer::new(&self.command_input);
        let tokens = l.tokenize()?;*/

        let mut p = Parser::new(&self.context.command);
        let action = p.parse()?;

        match &action {
            Action::FixtureSelector(fixture_selector) => {
                self.context.global_fixture_select = Some(fixture_selector.clone());
            }
            Action::ClearAll => {
                self.context.global_fixture_select = None;
            }
            Action::Test(cmd) => {
                match cmd.as_str() {
                    "start" => {
                        println!("starting seq");
                        let seq = self.context.preset_handler.sequence(1).unwrap().clone();
                        self.context
                            .sequence_runtimes
                            .push(SequenceRuntime::new(seq));

                        self.context.sequence_runtimes.last_mut().unwrap().play();
                    }
                    "next" => {
                        println!("going to next cue");
                        self.context
                            .sequence_runtimes
                            .last_mut()
                            .unwrap()
                            .next_cue();
                    }
                    _ => {}
                }
                self.context.preset_handler.sequence(1).unwrap();
            }
            _ => {}
        }

        let now = std::time::Instant::now();

        action.run(
            &mut self.context.fixture_handler,
            &mut self.context.preset_handler,
        )?;

        println!(
            "Execution of action {:?} took {:.2?}",
            action,
            now.elapsed()
        );

        Ok(())
    }
}

impl eframe::App for DemexUiApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        let now = std::time::Instant::now();

        // update fixture handler
        let _ = self
            .context
            .fixture_handler
            .update(&self.context.preset_handler);

        // update sequence runtimes
        for sr in self.context.sequence_runtimes.iter_mut() {
            sr.update(
                &mut self.context.fixture_handler,
                &self.context.preset_handler,
            );
        }

        let elapsed = now.elapsed();

        if self.max_update_time.is_none() || (self.max_update_time.unwrap() < elapsed) {
            self.max_update_time = Some(elapsed);
            println!("New max: {:?}", elapsed);
        }

        eframe::egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("demex");
                ui.separator();
                let slider =
                    ui.add(eframe::egui::Slider::new(&mut self.gm_slider_val, 0..=255).text("GM"));

                if slider.changed() {
                    *self.context.fixture_handler.grand_master_mut() = self.gm_slider_val;
                }
            });

            if let Some(error) = &self.global_error {
                ui.colored_label(eframe::egui::Color32::RED, format!("Error: {}", error));
            }
        });

        eframe::egui::CentralPanel::default().show(ctx, |ui| {
            self.tabs.ui(ui, &mut self.context, ctx);
        });

        eframe::egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.add_space(10.0);

            let command_label = ui.label("Command");

            let command_font = eframe::egui::FontId::new(16.0, eframe::egui::FontFamily::Monospace);

            ui.horizontal(|ui| {
                if !self.context.command.is_empty() {
                    ui.label(
                        eframe::egui::RichText::new(
                            self.context.command.iter().map(|t| t.to_string()).join(" "),
                        )
                        .background_color(eframe::egui::Color32::BLACK)
                        .color(eframe::egui::Color32::YELLOW)
                        .font(command_font.clone()),
                    );
                }

                let command_input_field = ui
                    .add_sized(
                        ui.available_size(),
                        eframe::egui::TextEdit::singleline(&mut self.command_input)
                            .font(command_font)
                            .text_color(eframe::egui::Color32::YELLOW),
                    )
                    .labelled_by(command_label.id);

                if command_input_field
                    .ctx
                    .input(|i| i.key_pressed(eframe::egui::Key::Space))
                {
                    let mut lexer = Lexer::new(&self.command_input);
                    let tokens = lexer.tokenize();

                    if let Ok(tokens) = tokens {
                        self.context
                            .command
                            .extend(tokens.iter().take(tokens.len() - 1).cloned());

                        self.command_input.clear();
                    }
                }

                if self.is_command_input_empty
                    && command_input_field
                        .ctx
                        .input(|i| i.key_pressed(eframe::egui::Key::Backspace))
                {
                    self.context.command.pop();
                }

                self.is_command_input_empty = self.command_input.is_empty();

                if !command_input_field.has_focus() {
                    command_input_field.request_focus();
                }

                if command_input_field
                    .ctx
                    .input(|i| i.key_pressed(eframe::egui::Key::Enter))
                {
                    let mut lexer = Lexer::new(&self.command_input);
                    let tokens = lexer.tokenize();

                    if let Ok(tokens) = tokens {
                        self.context.command.extend(tokens);

                        self.command_input.clear();

                        if let Err(e) = self.run_cmd() {
                            eprintln!("{}", e);
                            self.global_error = Some(e);
                        }

                        self.context.command.clear();
                    }
                }
            });
        });
    }
}
