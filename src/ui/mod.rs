use std::{sync::Arc, thread, time};

use command::ui_command_input;
use components::button::icon::DemexIcon;
use context::DemexUiContext;
use dlog::{DemexLogEntry, DemexLogEntryType};
use egui::IconData;
use strum::IntoEnumIterator;
use window::DemexWindow;

#[allow(unused_imports)]
use crate::{fixture::handler::FixtureHandler, lexer::Lexer};
use crate::{
    parser::nodes::{
        action::{Action, ConfigTypeActionData},
        fixture_selector::FixtureSelectorContext,
    },
    show::ui::DemexShowUiConfig,
    ui::viewport::position::DemexViewportPositonState,
    utils::version::VERSION_STR,
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
pub mod viewport;
pub mod window;

const UI_THREAD_NAME: &str = "demex-ui";

pub struct DemexUiApp {
    context: DemexUiContext,

    ui_config: DemexShowUiConfig,
    num_additional_viewports: usize,

    last_update: std::time::Instant,

    desired_fps: f64,

    icon: Arc<IconData>,

    is_single_threaded: bool,
    last_single_threaded_update: std::time::Instant,
}

impl DemexUiApp {
    pub fn new(
        context: DemexUiContext,
        desired_fps: f64,
        icon: Arc<IconData>,
        is_single_threaded: bool,
        num_additional_viewports: Option<usize>,
    ) -> Self {
        context
            .stats
            .write()
            .register_thread(UI_THREAD_NAME.to_owned(), thread::current().id());

        Self {
            ui_config: context.ui_config.clone(),

            last_update: time::Instant::now(),
            desired_fps,

            icon,

            is_single_threaded,
            last_single_threaded_update: time::Instant::now(),

            num_additional_viewports: num_additional_viewports.unwrap_or(0),

            context,
        }
    }
}

impl DemexUiApp {
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
        updatable_handler.update_executors(
            patch.fixture_types(),
            &fixture_handler,
            &preset_handler,
            &timing_handler,
        );

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
            &mut self.context.action_queue,
            &mut self.context.global_fixture_select,
            &mut self.context.command,
        ) {
            self.context
                .logs
                .push(DemexLogEntry::new(DemexLogEntryType::Error(
                    input_error.to_string(),
                )));
        }

        self.context.execute_action_queue(&self.ui_config);

        self.context.window_handler.show(
            ctx,
            &mut self.context.fixture_handler,
            &mut self.context.preset_handler,
            &mut self.context.updatable_handler,
            &mut self.context.patch,
        );

        for (idx, viewport) in self
            .ui_config
            .viewports
            .iter_mut()
            .skip(1)
            .take(self.num_additional_viewports)
            .filter(|viewport| viewport.should_render())
            .enumerate()
        {
            let viewport_title = format!("demex - Additional Viewport {}", idx + 1);
            let viewport_id = egui::ViewportId::from_hash_of(viewport_title.as_str());

            let mut viewport_builder = egui::ViewportBuilder::default()
                .with_title(&viewport_title)
                .with_icon(self.icon.clone());

            if let DemexViewportPositonState::Initial(rect) = viewport.pos_state() {
                viewport_builder = viewport_builder
                    .with_position(rect.min)
                    .with_inner_size(rect.size());
            }

            ctx.show_viewport_immediate(viewport_id, viewport_builder, |ctx, _| {
                let pos = ctx.input(|reader| reader.viewport().outer_rect.map(|r| r.min));
                let size = ctx.input(|reader| reader.viewport().outer_rect.map(|r| r.size()));

                if let (Some(pos), Some(size)) = (pos, size) {
                    *viewport.pos_state_mut() =
                        DemexViewportPositonState::Rendered(egui::Rect::from_min_size(pos, size));
                }

                egui::CentralPanel::default().show(ctx, |ui| {
                    viewport.ui(ui, &mut self.context);
                });
            });
        }

        eframe::egui::TopBottomPanel::top("DemexMainViewport").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("demex");

                ui.separator();

                ui.label(format!("v{}", VERSION_STR));

                ui.separator();

                if ui.link("Matteo Lutz").clicked() {
                    self.context.action_queue.enqueue_now(Action::MatteoLutz);
                }

                ui.separator();

                if ui.link("About demex").clicked() {
                    self.context
                        .window_handler
                        .add_window(DemexWindow::AboutDemex);
                }

                ui.separator();

                ui.menu_image_button(
                    DemexIcon::Draft
                        .button_image()
                        .tint(if self.context.show_file.is_some() {
                            ecolor::Color32::WHITE
                        } else {
                            ecolor::Color32::YELLOW
                        }),
                    |ui| {
                        if let Some(show_file) = self.context.show_file.as_ref() {
                            ui.label(show_file.display().to_string());
                        } else {
                            ui.colored_label(ecolor::Color32::YELLOW, "Show not saved");
                        }

                        ui.separator();

                        if ui.button("Save").clicked() {
                            ui.close_menu();
                            self.context.save_show(self.ui_config.clone());
                        }

                        if ui.button("Open").clicked() {
                            ui.close_menu();
                            self.context.open_new_show();
                        }
                    },
                );

                ui.menu_image_button(DemexIcon::Settings.button_image(), |ui| {
                    ui.menu_button("Config", |ui| {
                        if ui.button("All").clicked() {
                            ui.close_menu();

                            self.context.window_handler.add_window(DemexWindow::Edit(
                                window::edit::DemexEditWindow::ConfigOverview,
                            ));
                        }

                        ui.separator();

                        for config_type in ConfigTypeActionData::iter() {
                            if ui.button(format!("{:?}", config_type)).clicked() {
                                ui.close_menu();

                                self.context.window_handler.add_window(DemexWindow::Edit(
                                    window::edit::DemexEditWindow::Config(config_type),
                                ));
                            }
                        }
                    });

                    ui.separator();

                    if ui.button("TOP SECRET").clicked() {
                        ui.close_menu();
                        let _ = open::that("https://youtu.be/dQw4w9WgXcQ");
                    }
                });
            });
        });

        ui_command_input(ctx, &mut self.context);

        eframe::egui::CentralPanel::default().show(ctx, |ui| {
            self.ui_config.viewports[0].ui(ui, &mut self.context);
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
