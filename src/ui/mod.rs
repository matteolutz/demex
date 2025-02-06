use std::{sync::Arc, time};

use error::DemexUiError;
use parking_lot::RwLock;
use tabs::{layout_view_tab::LayoutViewContext, DemexTabs};

#[allow(unused_imports)]
use crate::{
    dmx::output::{debug_dummy::DebugDummyOutput, dmx_serial::DMXSerialOutput},
    fixture::{handler::FixtureHandler, Fixture},
    lexer::Lexer,
    parser::Parser,
};
use crate::{
    fixture::{
        channel::{value::FixtureChannelValue, FIXTURE_CHANNEL_INTENSITY_ID},
        effect::FixtureChannelEffect,
        patch::Patch,
        presets::PresetHandler,
        updatables::UpdatableHandler,
    },
    lexer::token::Token,
    parser::nodes::{
        action::Action,
        fixture_selector::{FixtureSelector, FixtureSelectorContext},
    },
};

pub mod components;
pub mod error;
pub mod graphics;
pub mod iimpl;
pub mod tabs;
pub mod traits;

#[derive(Debug, Default)]
pub struct DemexUiStats {
    pub ui_dt: f64,
    pub ui_max_dt: f64,

    pub fixed_update_dt: f64,
    pub fixed_update_max_dt: f64,
}

impl DemexUiStats {
    pub fn ui(&mut self, dt: f64) {
        self.ui_dt = dt;
        self.ui_max_dt = self.ui_max_dt.max(dt);
    }

    pub fn fixed_update(&mut self, dt: f64) {
        self.fixed_update_dt = dt;
        self.fixed_update_max_dt = self.fixed_update_max_dt.max(dt);
    }
}

pub struct DemexUiContext {
    patch: Patch,

    fixture_handler: Arc<RwLock<FixtureHandler>>,
    preset_handler: Arc<RwLock<PresetHandler>>,
    updatable_handler: Arc<RwLock<UpdatableHandler>>,

    global_fixture_select: Option<FixtureSelector>,
    command: Vec<Token>,

    stats: Arc<RwLock<DemexUiStats>>,

    layout_view_context: LayoutViewContext,
    gm_slider_val: u8,
}

pub struct DemexUiApp {
    command_input: String,
    is_command_input_empty: bool,
    context: DemexUiContext,
    global_error: Option<Box<dyn std::error::Error>>,
    tabs: DemexTabs,
    last_update: std::time::Instant,
    fps: f64,
}

impl DemexUiApp {
    pub fn new(
        fixture_handler: Arc<RwLock<FixtureHandler>>,
        preset_handler: Arc<RwLock<PresetHandler>>,
        updatable_handler: Arc<RwLock<UpdatableHandler>>,
        patch: Patch,
        stats: Arc<RwLock<DemexUiStats>>,
        fps: f64,
    ) -> Self {
        Self {
            command_input: String::new(),
            is_command_input_empty: true,
            context: DemexUiContext {
                patch,
                stats,
                gm_slider_val: fixture_handler.clone().read().grand_master(),
                fixture_handler,
                preset_handler,
                updatable_handler,
                global_fixture_select: None,
                command: Vec::new(),
                layout_view_context: LayoutViewContext::default(),
            },
            global_error: None,
            tabs: DemexTabs::default(),
            last_update: time::Instant::now(),
            fps,
        }
    }
}

impl DemexUiApp {
    pub fn run_and_handle_action(
        &mut self,
        action: Action,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match &action {
            Action::FixtureSelector(fixture_selector) => {
                let fixture_selector = fixture_selector.flatten(
                    &self.context.preset_handler.read(),
                    FixtureSelectorContext::new(&self.context.global_fixture_select),
                );

                if let Ok(fixture_selector) = fixture_selector {
                    let selected_fixtures = fixture_selector.get_fixtures(
                        &self.context.preset_handler.read(),
                        FixtureSelectorContext::new(&self.context.global_fixture_select),
                    );

                    if selected_fixtures.is_ok() {
                        self.context.global_fixture_select = Some(fixture_selector);
                    } else if let Err(fixture_selector_err) = selected_fixtures {
                        self.context.global_fixture_select = None;
                        self.global_error = Some(Box::new(fixture_selector_err));
                    }
                } else if let Err(fixture_selector_err) = fixture_selector {
                    self.context.global_fixture_select = None;
                    self.global_error = Some(Box::new(fixture_selector_err))
                }
            }
            Action::ClearAll => {
                self.context.global_fixture_select = None;
            }
            Action::GoHomeAll => {
                let mut updatable_handler_lock = self.context.updatable_handler.write();
                let mut fixture_handler_lock = self.context.fixture_handler.write();
                let preset_handler_lock = self.context.preset_handler.read();

                updatable_handler_lock
                    .sequence_runtimes_stop_all(&mut fixture_handler_lock, &preset_handler_lock);

                updatable_handler_lock.faders_home_all(&mut fixture_handler_lock);
            }
            Action::Test(cmd) => match cmd.as_str() {
                "effect" => {
                    let _ = self
                        .context
                        .fixture_handler
                        .write()
                        .fixture(1)
                        .unwrap()
                        .set_channel_value(
                            FIXTURE_CHANNEL_INTENSITY_ID,
                            FixtureChannelValue::Effect(FixtureChannelEffect::SingleSine {
                                a: 1.0,
                                b: 20.0,
                                c: 1.0,
                                d: 1.0,
                            }),
                        );
                }
                _ => {
                    self.global_error = Some(Box::new(DemexUiError::RuntimeError(format!(
                        "Unknown test command: \"{}\"",
                        cmd
                    ))))
                }
            },
            _ => {}
        }

        let now = std::time::Instant::now();

        action.run(
            &mut self.context.fixture_handler.write(),
            &mut self.context.preset_handler.write(),
            FixtureSelectorContext::new(&self.context.global_fixture_select),
            &self.context.updatable_handler.read(),
        )?;

        println!(
            "Execution of action {:?} took {:.2?}",
            action,
            now.elapsed()
        );

        Ok(())
    }

    pub fn run_cmd(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut p = Parser::new(&self.context.command);
        let action = p.parse()?;

        self.run_and_handle_action(action)
    }
}

impl eframe::App for DemexUiApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        if self.global_error.is_some() && ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.global_error = None;
        }

        if let Some(global_error) = &self.global_error {
            egui::Window::new("Error")
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .collapsible(false)
                .resizable(false)
                .interactable(false)
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(
                            egui::RichText::from(global_error.to_string())
                                .text_style(egui::TextStyle::Heading)
                                .color(egui::Color32::LIGHT_RED),
                        );
                    });
                });
        }

        /*egui::Window::new("Settings").show(ctx, |ui| {
            Probe::new(self.context.preset_handler.fader_mut(2).unwrap()).show(ui);
        });*/

        eframe::egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("demex");
                ui.separator();

                let slider = ui.add(
                    eframe::egui::Slider::new(&mut self.context.gm_slider_val, 0..=255).text("GM"),
                );

                if slider.changed() {
                    *self.context.fixture_handler.write().grand_master_mut() =
                        self.context.gm_slider_val;
                }

                ui.separator();

                if ui.button("Clear Selection").clicked() {
                    self.context.global_fixture_select = None;
                }
            });

            if let Some(error) = &self.global_error {
                ui.colored_label(
                    eframe::egui::Color32::LIGHT_RED,
                    format!("Error: {}", error),
                );
            }
        });

        eframe::egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.add_space(10.0);

            let command_label = ui.label("Command");

            let command_font = eframe::egui::FontId::new(16.0, eframe::egui::FontFamily::Monospace);

            ui.horizontal(|ui| {
                if !self.context.command.is_empty() {
                    ui.horizontal(|ui| {
                        for token in &self.context.command {
                            ui.label(
                                eframe::egui::RichText::from(token.to_string())
                                    .background_color(eframe::egui::Color32::BLACK)
                                    .color(token)
                                    .font(command_font.clone()),
                            );
                        }
                    });
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
                    if command_input_field
                        .ctx
                        .input(|i| i.modifiers.ctrl || i.modifiers.mac_cmd)
                    {
                        self.context.command.clear();
                    } else {
                        self.context.command.pop();
                    }
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

        eframe::egui::CentralPanel::default().show(ctx, |ui| {
            self.tabs.ui(ui, &mut self.context, ctx);
        });

        let elapsed = self.last_update.elapsed();

        self.context.stats.write().ui(elapsed.as_secs_f64());

        let _diff = f64::max((1.0 / self.fps) - elapsed.as_secs_f64(), 0.0);
        // ctx.request_repaint_after(time::Duration::from_secs_f64(diff));

        ctx.request_repaint();

        self.last_update = time::Instant::now();
    }
}
