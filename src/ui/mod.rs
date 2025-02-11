use std::{collections::HashSet, sync::Arc, thread, time};

use constants::VERSION_STR;
use context::DemexUiContext;
use log::{dialog::DemexGlobalDialogEntry, DemexLogEntry, DemexLogEntryType};
use parking_lot::RwLock;
use tabs::{layout_view_tab::LayoutViewContext, DemexTab, DemexTabs};
use window::DemexWindow;

#[allow(unused_imports)]
use crate::{
    fixture::{handler::FixtureHandler, Fixture},
    lexer::Lexer,
};
use crate::{
    fixture::{presets::PresetHandler, updatables::UpdatableHandler},
    parser::{nodes::action::Action, Parser2},
    show::DemexShow,
    utils::thread::DemexThreadStatsHandler,
};

pub mod components;
pub mod constants;
pub mod context;
pub mod error;
pub mod graphics;
pub mod iimpl;
pub mod log;
pub mod tabs;
pub mod traits;
pub mod utils;
pub mod window;

const UI_THREAD_NAME: &str = "demex-ui";

pub struct DemexUiApp {
    command_input: String,
    is_command_input_empty: bool,
    context: DemexUiContext,

    tabs: DemexTabs,
    detached_tabs: HashSet<DemexTab>,

    last_update: std::time::Instant,

    desired_fps: f64,
}

impl DemexUiApp {
    pub fn new(
        fixture_handler: Arc<RwLock<FixtureHandler>>,
        preset_handler: Arc<RwLock<PresetHandler>>,
        updatable_handler: Arc<RwLock<UpdatableHandler>>,
        stats: Arc<RwLock<DemexThreadStatsHandler>>,
        save_show: fn(DemexShow) -> Result<(), Box<dyn std::error::Error>>,
        desired_fps: f64,
    ) -> Self {
        stats
            .write()
            .register_thread(UI_THREAD_NAME.to_owned(), thread::current().id());

        Self {
            command_input: String::new(),
            is_command_input_empty: true,
            context: DemexUiContext {
                stats,
                gm_slider_val: FixtureHandler::default_grandmaster_value(),
                fixture_handler,
                preset_handler,
                updatable_handler,
                global_fixture_select: None,
                command: Vec::new(),
                layout_view_context: LayoutViewContext::default(),
                macro_execution_queue: Vec::new(),
                save_show,
                logs: Vec::new(),
                windows: Vec::new(),
            },
            tabs: DemexTabs::default(),
            detached_tabs: HashSet::new(),
            last_update: time::Instant::now(),
            desired_fps,
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
        for i in 0..self.context.windows.len() {
            if self.context.windows[i].ui(
                ctx,
                &mut self.context.fixture_handler,
                &mut self.context.preset_handler,
                &mut self.context.updatable_handler,
            ) {
                self.context.windows.remove(i);
            }
        }

        while !self.context.macro_execution_queue.is_empty() {
            let action = self.context.macro_execution_queue.remove(0);

            if let Err(e) = self.context.run_and_handle_action(&action) {
                eprintln!("{}", e);

                self.context
                    .add_dialog_entry(DemexGlobalDialogEntry::error(e.as_ref()));
            }
        }

        for detached_tab in self.detached_tabs.clone() {
            let tab_title = detached_tab.to_string();

            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of(tab_title.as_str()),
                egui::ViewportBuilder::default()
                    .with_title(format!("demex - {}", tab_title))
                    .with_maximized(true)
                    .with_window_level(egui::WindowLevel::AlwaysOnTop),
                |ctx, _| {
                    if ctx.input(|reader| reader.viewport().close_requested()) {
                        self.detached_tabs.remove(&detached_tab);
                        self.tabs.re_attach(detached_tab);
                    }

                    egui::CentralPanel::default().show(ctx, |ui| {
                        egui::ScrollArea::both().show(ui, |ui| {
                            detached_tab.ui(ui, &mut self.context);
                        });
                    });
                },
            );
        }

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

            ui.horizontal(|ui| {
                ui.label(format!("v{}", VERSION_STR));

                ui.separator();

                if ui.link("Matteo Lutz").clicked() {
                    let _ = self.context.run_and_handle_action(&Action::MatteoLutz);
                }

                ui.separator();

                if ui.link("About demex").clicked()
                    && !self.context.windows.contains(&DemexWindow::AboutDemex)
                {
                    self.context.windows.push(DemexWindow::AboutDemex);
                }
            });
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

                if self.context.windows.is_empty() {
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
                }
            });
        });

        eframe::egui::CentralPanel::default().show(ctx, |ui| {
            self.tabs
                .ui(ui, &mut self.context, ctx, &mut self.detached_tabs);
        });

        let elapsed = self.last_update.elapsed().as_secs_f64();
        let epxected_elapsed: f64 = 1.0 / self.desired_fps;
        let diff = epxected_elapsed - elapsed;

        if diff > 0.0 {
            std::thread::sleep(time::Duration::from_secs_f64(diff));
        }

        self.context
            .stats
            .write()
            .update(UI_THREAD_NAME, self.last_update.elapsed().as_secs_f64());
        self.last_update = time::Instant::now();

        ctx.request_repaint();
    }
}
