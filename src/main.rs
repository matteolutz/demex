#![warn(unused_extern_crates)]

pub mod color;
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

use gdtf::GdtfFile;
use headless::{
    controller::DemexHeadlessConroller, id::DemexProtoDeviceId, node::DemexHeadlessNode,
};
use itertools::Itertools;
use parking_lot::RwLock;
use show::{context::ShowContext, DemexShow};

use ui::utils::load::load_textures;
#[cfg(feature = "ui")]
use ui::{
    context::DemexUiContext, theme::DemexUiTheme, theme::DemexUiThemeAttribute,
    utils::icon::load_icon, DemexUiApp,
};

use utils::{
    deadlock::start_deadlock_checking_thread,
    thread::{demex_update_thread, DemexThreadStatsHandler},
};

use clap::Parser;

#[cfg(not(feature = "ui"))]
#[derive(Debug, Copy, Clone, PartialEq, Eq, clap::ValueEnum)]
enum DemexUiThemeAttribute {}

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

    /// Set a manual node id for the headless node.
    #[arg(long, value_name = "ID")]
    headless_id: Option<u32>,

    /// Set the UI theme to use. This is only used if the UI feature is enabled.
    #[arg(long, value_name = "THEME", conflicts_with = "headless")]
    ui_theme: Option<DemexUiThemeAttribute>,

    /// Number of additional viewports to create in the UI. This is only used if the UI feature is enabled.
    #[arg(
        short,
        long,
        value_name = "ADDITIONAL_VIEWPORTS",
        conflicts_with = "headless"
    )]
    additional_viewports: Option<usize>,

    /// Fullscreen all viewports in the UI. This is only used if the UI feature is enabled.
    #[arg(long, conflicts_with = "headless")]
    fullscreen: bool,
}

const TEST_MAX_FUPS: f64 = 60.0;
const TEST_MAX_DMX_FPS: f64 = 30.0;
const TEST_UI_FPS: f64 = 60.0;

const APP_ID: &str = "demex";

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
        if args.headless.is_some() {
            DemexProtoDeviceId::Node(args.headless_id.unwrap_or_default())
        } else {
            DemexProtoDeviceId::Controller
        },
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

    if args.headless.is_none() {
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
                updatable_handler.update_executors(
                    patch.fixture_types(),
                    &fixture_handler,
                    &preset_handler,
                    &timing_handler,
                );
            },
        );
    }

    if let Some(master_ip) = args.headless {
        log::info!("Running in headless mode, no UI will be shown");
        DemexHeadlessNode::default().start_headless_in_current_thread(
            master_ip,
            args.headless_id.unwrap_or_default(),
            context.clone(),
        )?;
    } else {
        DemexHeadlessConroller::default().start_controller_thread(stats.clone(), context.clone());

        #[cfg(feature = "ui")]
        {
            let icon = Arc::new(load_icon());

            log::info!("Starting UI fullscreen: {}", args.fullscreen);

            let mut viewport_builder = eframe::egui::ViewportBuilder::default()
                .with_maximized(true)
                .with_icon(icon.clone());
            if args.fullscreen {
                viewport_builder = viewport_builder.with_fullscreen(true);
            }

            let options = eframe::NativeOptions {
                viewport: viewport_builder,
                ..Default::default()
            };

            eframe::run_native(
                APP_ID,
                options,
                Box::new(|creation_context| {
                    egui_extras::install_image_loaders(&creation_context.egui_ctx);

                    let style = egui::Style {
                        visuals: egui::Visuals::dark(),
                        ..egui::Style::default()
                    };

                    creation_context.egui_ctx.set_style(style);
                    creation_context
                        .egui_ctx
                        .set_fonts(ui::utils::load::load_fonts());

                    args.ui_theme
                        .map(DemexUiTheme::from)
                        .unwrap_or(DemexUiTheme::Default)
                        .apply(&creation_context.egui_ctx);

                    if args.touchscreen_mode {
                        creation_context.egui_ctx.style_mut(|style| {
                            style.spacing.button_padding = emath::vec2(10.0, 10.0);

                            style.spacing.indent = 18.0 * 2.0;
                            style.spacing.icon_width = 14.0 * 2.0;
                            style.spacing.icon_width_inner = 8.0 * 2.0;

                            // DEFAULT: style.spacing.interact_size = [40.0, 18.0];
                            //
                            style.spacing.interact_size = emath::vec2(40.0, 18.0) * 1.5;
                            style.spacing.slider_rail_height = 8.0 * 2.0;
                            style.spacing.slider_width = 100.0 * 1.5;
                        });
                    }

                    let ui_app_state = DemexUiApp::new(
                        DemexUiContext::load_show(
                            &context,
                            show.input_device_configs,
                            show.ui_config,
                            args.show,
                            stats,
                            load_textures(&creation_context.egui_ctx),
                        ),
                        TEST_UI_FPS,
                        icon,
                        false,
                        args.additional_viewports,
                        args.fullscreen,
                    );

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
