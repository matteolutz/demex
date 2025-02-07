use std::{fmt, sync::Arc, time};

use error::DemexUiError;
use log::{DemexLogEntry, DemexLogEntryType};
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
    parser::{
        mod2::Parser2,
        nodes::{
            action::{result::ActionRunResult, Action},
            fixture_selector::{FixtureSelector, FixtureSelectorContext},
        },
    },
    show::DemexShow,
};

pub mod components;
pub mod error;
pub mod graphics;
pub mod iimpl;
pub mod log;
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

    dialogs: Vec<DemexGlobalDialogEntry>,
    logs: Vec<DemexLogEntry>,

    macro_execution_queue: Vec<Action>,

    save_show: fn(DemexShow) -> Result<(), Box<dyn std::error::Error>>,

    layout_view_context: LayoutViewContext,
    gm_slider_val: u8,
}

impl DemexUiContext {
    pub fn add_dialog_entry(&mut self, dialog_entry: DemexGlobalDialogEntry) {
        self.dialogs.push(dialog_entry.clone());
        self.logs
            .push(DemexLogEntry::new(log::DemexLogEntryType::DialogEntry(
                dialog_entry,
            )));
    }

    pub fn run_and_handle_action(
        &mut self,
        action: &Action,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match &action {
            Action::FixtureSelector(fixture_selector) => {
                let fixture_selector = fixture_selector.flatten(
                    &self.preset_handler.read(),
                    FixtureSelectorContext::new(&self.global_fixture_select),
                );

                if let Ok(fixture_selector) = fixture_selector {
                    let selected_fixtures = fixture_selector.get_fixtures(
                        &self.preset_handler.read(),
                        FixtureSelectorContext::new(&self.global_fixture_select),
                    );

                    if selected_fixtures.is_ok() {
                        self.global_fixture_select = Some(fixture_selector);
                    } else if let Err(fixture_selector_err) = selected_fixtures {
                        self.global_fixture_select = None;
                        self.dialogs
                            .push(DemexGlobalDialogEntry::error(&fixture_selector_err));
                    }
                } else if let Err(fixture_selector_err) = fixture_selector {
                    self.global_fixture_select = None;
                    self.dialogs
                        .push(DemexGlobalDialogEntry::error(&fixture_selector_err));
                }
            }
            Action::ClearAll => {
                self.global_fixture_select = None;
                self.dialogs.clear();
            }
            Action::HomeAll => {
                let mut updatable_handler_lock = self.updatable_handler.write();
                let mut fixture_handler_lock = self.fixture_handler.write();
                let preset_handler_lock = self.preset_handler.read();

                updatable_handler_lock
                    .executors_stop_all(&mut fixture_handler_lock, &preset_handler_lock);

                updatable_handler_lock.faders_home_all(&mut fixture_handler_lock);
            }
            Action::Save => {
                let updatable_handler_lock = self.updatable_handler.read();
                let preset_handler_lock = self.preset_handler.read();

                let show = DemexShow {
                    preset_handler: preset_handler_lock.clone(),
                    updatable_handler: updatable_handler_lock.clone(),
                };

                let save_result = (self.save_show)(show);

                std::mem::forget(updatable_handler_lock);
                std::mem::forget(preset_handler_lock);

                if let Err(e) = save_result {
                    self.add_dialog_entry(DemexGlobalDialogEntry::error(e.as_ref()));
                } else {
                    self.add_dialog_entry(DemexGlobalDialogEntry::info("Show saved"));
                }
            }
            Action::Test(cmd) => match cmd.as_str() {
                "effect" => {
                    let _ = self
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
                _ => self.add_dialog_entry(DemexGlobalDialogEntry::error(
                    &DemexUiError::RuntimeError(format!("Unknown test command: \"{}\"", cmd)),
                )),
            },
            _ => {}
        }

        let now = std::time::Instant::now();

        let result = action
            .run(
                &mut self.fixture_handler.write(),
                &mut self.preset_handler.write(),
                FixtureSelectorContext::new(&self.global_fixture_select),
                &mut self.updatable_handler.write(),
            )
            .inspect(|result| {
                self.logs.push(DemexLogEntry::new(
                    log::DemexLogEntryType::ActionEntrySuccess(action.clone(), result.clone()),
                ))
            })
            .inspect_err(|err| {
                self.logs.push(DemexLogEntry::new(
                    log::DemexLogEntryType::ActionEntryFailed(action.clone(), err.to_string()),
                ))
            })?;

        println!(
            "Execution of action {:?} took {:.2?}",
            action,
            now.elapsed()
        );

        match result {
            ActionRunResult::Warn(warn) => {
                self.add_dialog_entry(DemexGlobalDialogEntry::warn(warn.as_str()));
            }
            ActionRunResult::Info(info) => {
                self.add_dialog_entry(DemexGlobalDialogEntry::info(info.as_str()));
            }
            _ => {}
        }

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum DemexGlobalDialogEntryType {
    Error,
    Warn,
    Info,
}

impl DemexGlobalDialogEntryType {
    pub fn title(&self) -> &str {
        match self {
            Self::Error => "Error",
            Self::Warn => "Warning",
            Self::Info => "Info",
        }
    }

    pub fn color(&self) -> egui::Color32 {
        match self {
            Self::Error => egui::Color32::LIGHT_RED,
            Self::Warn => egui::Color32::YELLOW,
            Self::Info => egui::Color32::LIGHT_BLUE,
        }
    }
}

#[derive(Clone)]
pub struct DemexGlobalDialogEntry {
    entry_type: DemexGlobalDialogEntryType,
    message: String,
    time: chrono::DateTime<chrono::Local>,
}

impl DemexGlobalDialogEntry {
    pub fn new(entry_type: DemexGlobalDialogEntryType, message: String) -> Self {
        Self {
            entry_type,
            message,
            time: chrono::offset::Local::now(),
        }
    }

    pub fn color(&self) -> egui::Color32 {
        self.entry_type.color()
    }

    pub fn time(&self) -> chrono::DateTime<chrono::Local> {
        self.time
    }

    pub fn error(error: &dyn std::error::Error) -> Self {
        Self::new(DemexGlobalDialogEntryType::Error, error.to_string())
    }

    pub fn warn(warn: &str) -> Self {
        Self::new(DemexGlobalDialogEntryType::Warn, warn.to_string())
    }

    pub fn info(info: &str) -> Self {
        Self::new(DemexGlobalDialogEntryType::Info, info.to_string())
    }
}

impl fmt::Display for DemexGlobalDialogEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {}: {}",
            self.time.format("%H:%M:%S"),
            self.entry_type.title(),
            self.message
        )
    }
}

pub struct DemexUiApp {
    command_input: String,
    is_command_input_empty: bool,
    context: DemexUiContext,

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
        save_show: fn(DemexShow) -> Result<(), Box<dyn std::error::Error>>,
    ) -> Self {
        Self {
            command_input: String::new(),
            is_command_input_empty: true,
            context: DemexUiContext {
                patch,
                stats,
                gm_slider_val: fixture_handler.clone().read().grand_master(),
                dialogs: Vec::new(),
                fixture_handler,
                preset_handler,
                updatable_handler,
                global_fixture_select: None,
                command: Vec::new(),
                layout_view_context: LayoutViewContext::default(),
                macro_execution_queue: Vec::new(),
                save_show,
                logs: Vec::new(),
            },
            tabs: DemexTabs::default(),
            last_update: time::Instant::now(),
            fps,
        }
    }
}

impl DemexUiApp {
    pub fn run_cmd(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.context
            .logs
            .push(DemexLogEntry::new(DemexLogEntryType::CommandEntry(
                self.context.command.clone(),
            )));

        let mut p = Parser2::new(&self.context.command);

        let action = p.parse().inspect_err(|err| {
            self.context
                .logs
                .push(DemexLogEntry::new(DemexLogEntryType::CommandFailedEntry(
                    err.to_string(),
                )))
        })?;

        self.context.run_and_handle_action(&action)
    }
}

impl eframe::App for DemexUiApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        while !self.context.macro_execution_queue.is_empty() {
            let action = self.context.macro_execution_queue.remove(0);

            if let Err(e) = self.context.run_and_handle_action(&action) {
                eprintln!("{}", e);

                self.context
                    .add_dialog_entry(DemexGlobalDialogEntry::error(e.as_ref()));
            }
        }

        if !self.context.dialogs.is_empty() && ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.context.dialogs.clear();
        }

        if !self.context.dialogs.is_empty() {
            egui::Window::new("demex dialog")
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .collapsible(false)
                .resizable(false)
                .interactable(false)
                .show(ctx, |ui| {
                    ui.vertical(|ui| {
                        for dialog in &self.context.dialogs {
                            ui.label(
                                egui::RichText::from(dialog.to_string())
                                    .strong()
                                    .color(dialog.color()),
                            );
                            ui.separator();
                        }
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

            let num_of_type =
                |dialogs: &Vec<DemexGlobalDialogEntry>, dialog_type: DemexGlobalDialogEntryType| {
                    dialogs
                        .iter()
                        .filter(|d| d.entry_type == dialog_type)
                        .count()
                };

            let num_of_errors =
                num_of_type(&self.context.dialogs, DemexGlobalDialogEntryType::Error);
            let num_of_warns = num_of_type(&self.context.dialogs, DemexGlobalDialogEntryType::Warn);
            let num_of_infos = num_of_type(&self.context.dialogs, DemexGlobalDialogEntryType::Info);

            let dialog_summary = format!(
                "Errors: {}, Warns: {}, Infos: {}",
                num_of_errors, num_of_warns, num_of_infos
            );

            ui.colored_label(egui::Color32::PLACEHOLDER, dialog_summary);
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
                            self.context
                                .add_dialog_entry(DemexGlobalDialogEntry::error(e.as_ref()));
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
