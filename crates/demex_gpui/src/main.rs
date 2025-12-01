#![warn(unused_extern_crates)]

pub mod engine;
pub mod storage;
pub mod utils;

#[cfg(feature = "ui")]
pub mod ui;

#[cfg(feature = "gpui")]
pub mod ui2;

use std::path::PathBuf;

use gdtf::GdtfFile;
use itertools::Itertools;

#[cfg(feature = "ui")]
use ui::{
    DemexUiApp, context::DemexUiContext, theme::DemexUiTheme, theme::DemexUiThemeAttribute,
    utils::icon::load_icon, utils::load::load_textures,
};

use demex_core::{show::DemexShow, utils::deadlock::start_deadlock_checking_thread};

use clap::Parser;

use crate::engine::DemexEngineHandler;

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

    /// Enable the controller mode, which allows the application to act as a controller for headless nodes.
    #[arg(long, default_value = "false", conflicts_with = "headless")]
    controller: bool,
}

const APP_ID: &str = "demex";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::var("RUST_LOG").is_err() {
        unsafe {
            std::env::set_var("RUST_LOG", "debug");
        }
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

    if let Some(_master_ip) = args.headless {
        log::info!("Running in headless mode, no UI will be shown");
        /*
        DemexHeadlessNode::default().start_headless_in_current_thread(
            master_ip,
            args.headless_id.unwrap_or_default(),
            context.clone(),
        )?;
        */
    } else {
        if args.controller {
            log::info!("Running in controller mode.");
            /*
            DemexHeadlessConroller::default().start_controller_thread(
                stats.clone(),
                context.clone(),
                udp_rx,
            );
            */
        }

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

            let ui_theme = args
                .ui_theme
                .map(DemexUiTheme::from)
                .unwrap_or(DemexUiTheme::Default);

            let options = ui_theme.native_options(viewport_builder);

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

                    ui_theme.apply(&creation_context.egui_ctx);

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
                            input_device_event_handler,
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

        #[cfg(feature = "gpui")]
        {
            use gpui_component_assets::Assets;

            gpui::Application::new()
                .with_assets(Assets)
                .run(|cx: &mut gpui::App| {
                    use gpui_component::{Theme, ThemeRegistry};

                    use crate::ui2::wm::{self, WindowManager};

                    gpui_component::init(cx);

                    let theme_reg = ThemeRegistry::global(cx);
                    if let Some(theme) = theme_reg.themes().get("Default Dark").cloned() {
                        Theme::global_mut(cx).apply_config(&theme);
                    }

                    cx.activate(true);

                    let wm = WindowManager::new(cx);
                    cx.set_global(wm);
                    wm::init(cx);

                    DemexEngineHandler::init(fixture_types, show, cx)
                        .expect("Failed to initialize engine");

                    cx.spawn(async move |cx| {
                        cx.open_window(Default::default(), |window, cx| {
                            use gpui::AppContext;
                            use gpui_component::Root;

                            use crate::ui2::pane::MainPane;

                            window.set_window_title("demex");
                            window.set_app_id(APP_ID);

                            let view = cx.new(|cx| MainPane::new(window, cx));
                            cx.new(|cx| Root::new(view, window, cx))
                        })?;

                        Ok::<_, anyhow::Error>(())
                    })
                    .detach();

                    /*
                    cx.update_wm(|wm, cx| wm.open_singleton_window::<MainWindow>(cx, ()));

                    cx.on_window_closed(|cx| {
                        if cx.windows().is_empty() {
                            cx.quit();
                        }
                    })
                    .detach();*/
                });
        }

        #[cfg(all(not(feature = "ui"), not(feature = "gpui")))]
        {
            log::error!(
                "UI feature is not enabled. Please enable the UI feature to run the application with a user interface or run in headless mode."
            );
            std::process::exit(1);
        }
    }

    Ok(())
}
