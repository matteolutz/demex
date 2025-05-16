#![warn(unused_extern_crates)]

pub mod dmx;
pub mod fixture;
pub mod headless;
pub mod input;
pub mod lexer;
pub mod parser;
pub mod show;
pub mod storage;

#[cfg(feature = "ui")]
pub mod ui;

pub mod utils;

use std::{path::PathBuf, sync::Arc, time};

use egui::{Style, Visuals};
use gdtf::GdtfFile;
use headless::{controller::DemexHeadlessConroller, node::DemexHeadlessNode};
use itertools::Itertools;
use parking_lot::RwLock;
use show::{context::ShowContext, DemexShow};

#[cfg(feature = "ui")]
use ui::{context::DemexUiContext, theme::DemexUiTheme, utils::icon::load_icon, DemexUiApp};

use utils::{
    deadlock::start_deadlock_checking_thread,
    thread::{demex_update_thread, DemexThreadStatsHandler},
};

use clap::Parser;

/// demex - command based stage lighting control
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to the show file to load
    #[arg(short, long, conflicts_with = "headless")]
    show: Option<PathBuf>,

    /// Run an additional thread to periodically check for RwLock deadlocks
    #[arg(long)]
    deadlock_test: bool,

    /// Run the application in a mode that is more suitable for touchscreen devices (i.e. larger UI elements, ..)
    #[arg(long, conflicts_with = "headless")]
    touchscreen_mode: bool,

    /// Run the application in headless mode (i.e. no UI, used for demex nodes). Pass the ip address of the controller node to connect to.
    #[arg(long, value_name = "IP_ADDRESS")]
    headless: Option<String>,
}

const TEST_MAX_FUPS: f64 = 60.0;
const TEST_MAX_DMX_FPS: f64 = 30.0;
const TEST_UI_FPS: f64 = 60.0;

#[cfg(feature = "ui")]
const TEST_UI_THEME: DemexUiTheme = DemexUiTheme::Default;

const APP_ID: &str = "demex";

fn load_fonts() -> egui::FontDefinitions {
    let mut fonts = egui::FontDefinitions::default();

    fonts.font_data.insert(
        "open-sans".to_string(),
        Arc::new(egui::FontData::from_static(include_bytes!(
            "../assets/fonts/OpenSans-Regular.ttf"
        ))),
    );
    fonts.font_data.insert(
        "jetbrains-mono".to_string(),
        Arc::new(egui::FontData::from_static(include_bytes!(
            "../assets/fonts/JetBrainsMono-Regular.ttf"
        ))),
    );

    fonts
        .families
        .get_mut(&egui::FontFamily::Proportional)
        .unwrap()
        .insert(0, "open-sans".to_string());

    fonts
        .families
        .get_mut(&egui::FontFamily::Monospace)
        .unwrap()
        .insert(0, "jetbrains-mono".to_string());

    fonts.families.insert(
        egui::FontFamily::Name("Timecode".into()),
        vec!["timecode".to_string()],
    );

    fonts
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "debug");
    }

    env_logger::init();
    log::info!("Starting demex");

    let args = Args::parse();

    if args.deadlock_test {
        start_deadlock_checking_thread();
    }

    let fixture_files = std::fs::read_dir(storage::fixture_types(APP_ID))
        .unwrap()
        .flat_map(|file| {
            file.ok()
                .and_then(|f| std::fs::File::open(f.path()).ok())
                .and_then(|f| GdtfFile::new(f).ok())
        })
        .collect::<Vec<_>>();

    log::info!(
        "Found {} valid fixtures file(s), with {} fixture type(s) (at {})",
        fixture_files.len(),
        fixture_files
            .iter()
            .map(|f| f.description.fixture_types.len())
            .sum::<usize>(),
        storage::fixture_types(APP_ID).display()
    );
    log::debug!(
        "Valid fixture type(s):\n {}",
        fixture_files
            .iter()
            .flat_map(|f| &f.description.fixture_types)
            .map(|fixture_type| format!(
                "{} (man.: {}, id: {}, modes: {:?})\n",
                fixture_type.long_name,
                fixture_type.manufacturer,
                fixture_type.fixture_type_id,
                fixture_type
                    .dmx_modes
                    .iter()
                    .map(|mode| &mode.name)
                    .collect::<Vec<_>>()
            ))
            .join(", ")
    );

    let show: DemexShow = args
        .show
        .as_ref()
        .inspect(|show_path| log::info!("Loading show file: {:?}", show_path))
        .map(|show_path| serde_json::from_reader(std::fs::File::open(show_path).unwrap()).unwrap())
        .unwrap_or(DemexShow::default());

    let fixture_types = fixture_files
        .into_iter()
        .flat_map(|file| file.description.fixture_types)
        .collect::<Vec<_>>();

    let stats = Arc::new(RwLock::new(DemexThreadStatsHandler::default()));
    let context = ShowContext::new(
        fixture_types,
        show.patch,
        show.preset_handler,
        show.updatable_handler,
        show.timing_handler,
        args.headless.is_some(),
    );

    let fixture_handler_thread_a = context.fixture_handler.clone();
    let preset_handler_thread_a = context.preset_handler.clone();
    let timing_handler_thread_a = context.timing_handler.clone();
    let patch_thread_a = context.patch.clone();

    demex_update_thread(
        "demex-dmx-output".to_owned(),
        stats.clone(),
        TEST_MAX_DMX_FPS,
        move |_, last_user_update| {
            let mut fixture_handler = fixture_handler_thread_a.write();
            let preset_handler = preset_handler_thread_a.read();
            let timing_handler = timing_handler_thread_a.read();
            let patch = patch_thread_a.read();

            if fixture_handler
                .generate_output_data(
                    patch.fixture_types(),
                    &preset_handler,
                    &timing_handler,
                    last_user_update.elapsed().as_secs_f64() > 1.0,
                )
                .inspect_err(|err| log::error!("Failed to generate output data: {}", err))
                .is_ok_and(|res| res > 0)
            {
                *last_user_update = time::Instant::now();
            }
        },
    );

    let fixture_handler_thread_b = context.fixture_handler.clone();
    let preset_handler_thread_b = context.preset_handler.clone();
    let updatable_handler_thread_b = context.updatable_handler.clone();
    let timing_handler_thread_b = context.timing_handler.clone();
    let patch_thread_b = context.patch.clone();

    demex_update_thread(
        "demex-update".to_owned(),
        stats.clone(),
        TEST_MAX_FUPS,
        move |_, _| {
            let mut fixture_handler = fixture_handler_thread_b.write();
            let preset_handler = preset_handler_thread_b.read();
            let mut updatable_handler = updatable_handler_thread_b.write();
            let timing_handler = timing_handler_thread_b.read();
            let patch = patch_thread_b.read();

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
        },
    );

    if let Some(master_ip) = args.headless {
        log::info!("Running in headless mode, no UI will be shown");
        DemexHeadlessNode::new().start_headless_in_current_thread(master_ip, context.clone())?;
    } else {
        DemexHeadlessConroller::new().start_controller_thread(stats.clone(), context.clone());

        #[cfg(feature = "ui")]
        {
            let icon = Arc::new(load_icon());

            let ui_app_state = DemexUiApp::new(
                DemexUiContext::load_show(
                    &context,
                    show.input_device_configs,
                    show.ui_config,
                    args.show,
                    stats,
                ),
                TEST_UI_FPS,
                icon.clone(),
                false,
            );

            let options = eframe::NativeOptions {
                viewport: eframe::egui::ViewportBuilder::default()
                    .with_maximized(true)
                    .with_icon(icon),
                ..Default::default()
            };

            eframe::run_native(
                APP_ID,
                options,
                Box::new(|creation_context| {
                    egui_extras::install_image_loaders(&creation_context.egui_ctx);

                    let style = Style {
                        visuals: Visuals::dark(),
                        ..Style::default()
                    };

                    creation_context.egui_ctx.set_style(style);
                    creation_context.egui_ctx.set_fonts(load_fonts());

                    TEST_UI_THEME.apply(&creation_context.egui_ctx);

                    if args.touchscreen_mode {
                        creation_context.egui_ctx.style_mut(|style| {
                            style.spacing.button_padding = egui::vec2(10.0, 10.0);

                            style.spacing.indent = 18.0 * 2.0;
                            style.spacing.icon_width = 14.0 * 2.0;
                            style.spacing.icon_width_inner = 8.0 * 2.0;

                            // DEFAULT: style.spacing.interact_size = [40.0, 18.0];
                            //
                            style.spacing.interact_size = egui::vec2(40.0, 18.0) * 1.5;
                            style.spacing.slider_rail_height = 8.0 * 2.0;
                            style.spacing.slider_width = 100.0 * 1.5;
                        });
                    }

                    Ok(Box::new(ui_app_state))
                }),
            )?;
        }

        #[cfg(not(feature = "ui"))]
        {
            log::error!("UI feature is not enabled. Please enable the UI feature to run the application with a user interface or run in headless mode.");
            std::process::exit(1);
        }
    }

    Ok(())
}
