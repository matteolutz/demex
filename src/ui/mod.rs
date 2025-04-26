use std::{path::PathBuf, sync::Arc, thread, time};

use command::ui_command_input;
use context::{DemexUiContext, SaveShowFn};
use dlog::{dialog::DemexGlobalDialogEntry, DemexLogEntry, DemexLogEntryType};
use egui::IconData;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tabs::{encoders_tab::EncodersTabState, DemexTabs};
use window::{DemexWindow, DemexWindowHandler};

#[allow(unused_imports)]
use crate::{fixture::handler::FixtureHandler, lexer::Lexer};
use crate::{
    fixture::{
        patch::Patch, presets::PresetHandler, timing::TimingHandler, updatables::UpdatableHandler,
    },
    input::DemexInputDeviceHandler,
    parser::{
        nodes::{action::Action, fixture_selector::FixtureSelectorContext},
        Parser2,
    },
    show::ui::DemexShowUiConfig,
    utils::{thread::DemexThreadStatsHandler, version::VERSION_STR},
};

pub mod command;
pub mod components;
pub mod constants;
pub mod context;
pub mod dlog;
pub mod error;
pub mod graphics;
pub mod iimpl;
pub mod patch;
pub mod tabs;
pub mod theme;
pub mod traits;
pub mod utils;
pub mod window;

const UI_THREAD_NAME: &str = "demex-ui";

#[derive(Default, Serialize, Deserialize, Debug, Copy, Clone)]
pub struct DetachedTabConfigPosSize {
    pos: egui::Pos2,
    size: egui::Vec2,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct DetachedTabConfig {
    is_fullscreen: bool,
    pos_size: Option<DetachedTabConfigPosSize>,

    #[serde(skip_serializing, skip_deserializing, default)]
    open: bool,
}

pub struct DemexUiApp {
    context: DemexUiContext,

    tabs: DemexTabs,

    command_auto_focus: bool,

    last_update: std::time::Instant,

    desired_fps: f64,

    icon: Arc<IconData>,

    is_single_threaded: bool,
    last_single_threaded_update: std::time::Instant,
}

impl DemexUiApp {
    pub fn new(
        fixture_handler: Arc<RwLock<FixtureHandler>>,
        preset_handler: Arc<RwLock<PresetHandler>>,
        updatable_handler: Arc<RwLock<UpdatableHandler>>,
        timing_handler: Arc<RwLock<TimingHandler>>,
        patch: Arc<RwLock<Patch>>,
        stats: Arc<RwLock<DemexThreadStatsHandler>>,
        show_file: Option<PathBuf>,
        save_show: SaveShowFn,
        desired_fps: f64,
        icon: Arc<IconData>,
        input_device_handler: DemexInputDeviceHandler,
        ui_config: DemexShowUiConfig,
        is_single_threaded: bool,
    ) -> Self {
        stats
            .write()
            .register_thread(UI_THREAD_NAME.to_owned(), thread::current().id());

        Self {
            context: DemexUiContext {
                stats,
                gm_slider_val: FixtureHandler::default_grandmaster_value(),
                fixture_handler,
                preset_handler,
                updatable_handler,
                timing_handler,
                patch,

                global_fixture_select: None,

                command: Vec::new(),
                macro_execution_queue: Vec::new(),

                show_file,
                save_show,

                logs: vec![
                    DemexLogEntry::new(DemexLogEntryType::Info(format!(
                        "demex v{} (by @matteolutz), Welcome!",
                        VERSION_STR
                    ))),
                    DemexLogEntry::new(DemexLogEntryType::Info(
                        "Check out https://demex.matteolutz.de to get started.".to_owned(),
                    )),
                ],
                window_handler: DemexWindowHandler::default(),

                command_input: String::new(),
                is_command_input_empty: true,

                input_device_handler,

                ui_config,

                encoders_tab_state: EncodersTabState::default(),
            },
            tabs: DemexTabs::default(),

            command_auto_focus: false,

            last_update: time::Instant::now(),
            desired_fps,

            icon,

            is_single_threaded,
            last_single_threaded_update: time::Instant::now(),
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

    pub fn update_single_threaded(&mut self) {
        let mut fixture_handler = self.context.fixture_handler.write();
        let preset_handler = self.context.preset_handler.read();
        let mut updatable_handler = self.context.updatable_handler.write();
        let timing_handler = self.context.timing_handler.read();
        let patch = self.context.patch.read();

        let _ = fixture_handler
            .update_output_values(
                patch.fixture_types(),
                &preset_handler,
                &updatable_handler,
                &timing_handler,
            )
            .inspect_err(|err| log::error!("Failed to update fixture handler: {}", err));
        updatable_handler.update_faders(&preset_handler);
        updatable_handler.update_executors(&mut fixture_handler, &preset_handler);

        if fixture_handler
            .generate_output_data(
                patch.fixture_types(),
                &preset_handler,
                &timing_handler,
                self.last_single_threaded_update.elapsed().as_secs_f64() > 1.0,
            )
            .inspect_err(|err| log::error!("Failed to generate output data: {}", err))
            .is_ok_and(|res| res > 0)
        {
            self.last_single_threaded_update = time::Instant::now();
        }
    }
}

impl eframe::App for DemexUiApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        if self.is_single_threaded {
            self.update_single_threaded();
        }

        if let Err(input_error) = self.context.input_device_handler.update(
            &mut self.context.fixture_handler.write(),
            &mut self.context.preset_handler.write(),
            &mut self.context.updatable_handler.write(),
            &mut self.context.timing_handler.write(),
            &self.context.patch.read(),
            FixtureSelectorContext::new(&self.context.global_fixture_select.clone()),
            &mut self.context.macro_execution_queue,
            &mut self.context.global_fixture_select,
            &mut self.context.command,
        ) {
            self.context
                .logs
                .push(DemexLogEntry::new(DemexLogEntryType::Error(
                    input_error.to_string(),
                )));
        }

        self.context.window_handler.show(
            ctx,
            &mut self.context.fixture_handler,
            &mut self.context.preset_handler,
            &mut self.context.updatable_handler,
            &mut self.context.patch,
        );

        while !self.context.macro_execution_queue.is_empty() {
            let action = self.context.macro_execution_queue.remove(0);

            if let Err(e) = self.context.run_and_handle_action(&action) {
                log::warn!("{}", e);

                self.context
                    .add_dialog_entry(DemexGlobalDialogEntry::error(e.as_ref()));
            }
        }

        for detached_tab in self.context.ui_config.detached_tabs.clone() {
            let tab_title = detached_tab.to_string();

            // get current tab config as mut reference
            // insert if it does not exist

            let tab_config = self
                .context
                .ui_config
                .detached_tabs_config
                .entry(detached_tab)
                .or_default();

            let viewport_id = egui::ViewportId::from_hash_of(tab_title.as_str());

            let mut viewport_builder = egui::ViewportBuilder::default()
                .with_title(format!("demex - {}", tab_title))
                .with_icon(self.icon.clone())
                .with_window_level(egui::WindowLevel::AlwaysOnTop)
                .with_fullscreen(tab_config.is_fullscreen);

            if let Some(pos_size) = tab_config.pos_size.as_ref() {
                if !tab_config.open {
                    viewport_builder = viewport_builder
                        .with_position(pos_size.pos)
                        .with_inner_size(pos_size.size);
                    tab_config.open = true;
                }
            }

            ctx.show_viewport_immediate(viewport_id, viewport_builder, |ctx, _| {
                let tab_config = self
                    .context
                    .ui_config
                    .detached_tabs_config
                    .entry(detached_tab)
                    .or_default();

                if ctx.input(|reader| reader.viewport().close_requested()) {
                    self.context.ui_config.detached_tabs.remove(&detached_tab);
                    self.tabs.re_attach(detached_tab);
                    tab_config.open = false;
                }

                let pos = ctx.input(|reader| reader.viewport().outer_rect.map(|r| r.min));
                let size = ctx.input(|reader| reader.viewport().outer_rect.map(|r| r.size()));

                if let (Some(pos), Some(size)) = (pos, size) {
                    tab_config.pos_size = Some(DetachedTabConfigPosSize { pos, size })
                }

                egui::TopBottomPanel::top(format!("DemexDetachedTab-{}", tab_title)).show(
                    ctx,
                    |ui| {
                        if ui
                            .button(
                                if ctx
                                    .input(|reader| reader.viewport().fullscreen.is_some_and(|f| f))
                                {
                                    "Exit Fullscreen"
                                } else {
                                    "Fullscreen"
                                },
                            )
                            .clicked()
                        {
                            tab_config.is_fullscreen = !tab_config.is_fullscreen;
                        }
                    },
                );

                ui_command_input(ctx, &mut self.context, self.command_auto_focus);

                egui::CentralPanel::default().show(ctx, |ui| {
                    egui::ScrollArea::both().auto_shrink(false).show(ui, |ui| {
                        detached_tab.ui(ui, &mut self.context);
                    });
                });
            });
        }

        eframe::egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("demex");

                ui.separator();

                ui.label(format!("v{}", VERSION_STR));

                ui.separator();

                if ui.link("Matteo Lutz").clicked() {
                    let _ = self.context.run_and_handle_action(&Action::MatteoLutz);
                }

                ui.separator();

                if ui.link("About demex").clicked() {
                    self.context
                        .window_handler
                        .add_window(DemexWindow::AboutDemex);
                }

                ui.separator();

                if let Some(show_file) = self.context.show_file.as_ref() {
                    ui.label(show_file.display().to_string());
                } else {
                    ui.colored_label(egui::Color32::YELLOW, "Show not saved");
                }

                ui.separator();

                ui.checkbox(&mut self.command_auto_focus, "CMD AF");
            });
        });

        ui_command_input(ctx, &mut self.context, self.command_auto_focus);

        eframe::egui::CentralPanel::default().show(ctx, |ui| {
            self.tabs.ui(ui, &mut self.context, ctx);
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
