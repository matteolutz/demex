#![warn(unused_extern_crates)]

pub mod engine;
pub mod storage;
pub mod utils;

pub mod ui2;

use std::path::PathBuf;

use gdtf::GdtfFile;
use itertools::Itertools;

use demex_core::{show::DemexShow, utils::deadlock::start_deadlock_checking_thread};

use clap::Parser;

use crate::engine::DemexEngineHandler;

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

        use crate::ui2::assets::Assets;

        mod actions {
            use crate::engine::DemexEngineHandler;
            use gpui::{App, KeyBinding, Menu, MenuItem, SystemMenuType};

            gpui::actions!(demex, [Quit, Save]);
            pub(super) fn init(cx: &mut App) {
                cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);
                cx.bind_keys([KeyBinding::new("secondary-s", Save, None)]);

                cx.on_action::<Quit>(|_, cx| cx.quit());
                cx.on_action::<Save>(|_, cx| DemexEngineHandler::save(cx));

                init_menus(cx);
            }

            pub fn init_menus(cx: &mut App) {
                cx.set_menus(vec![Menu {
                    name: "demex".into(),
                    items: vec![
                        MenuItem::os_submenu("Services", SystemMenuType::Services),
                        MenuItem::separator(),
                        MenuItem::action("Quit", Quit),
                    ],
                }]);
            }
        }

        gpui::Application::new()
            .with_assets(Assets)
            .run(move |cx: &mut gpui::App| {
                use gpui_component::{Theme, ThemeRegistry};

                use crate::ui2::{
                    config::DemexUiConfig,
                    wm::{WindowManager, app::WindowManagerAppExt, dock_window::DockWindowConfig},
                };

                gpui_component::init(cx);
                ui2::init(cx).unwrap();

                actions::init(cx);

                let theme_reg = ThemeRegistry::global(cx);
                if let Some(theme) = theme_reg.themes().get("Default Dark").cloned() {
                    Theme::global_mut(cx).apply_config(&theme);
                }

                cx.activate(true);

                let ui_config = DemexUiConfig {
                    touchcreen_mode: args.touchscreen_mode,
                };
                cx.set_global(ui_config);

                DemexEngineHandler::init(fixture_types, show, cx)
                    .expect("Failed to initialize engine");

                let wm = WindowManager::new(cx).auto_quit(true);
                cx.set_global(wm);

                cx.update_wm(|wm, cx| wm.add_dock_window(DockWindowConfig::default(), cx));
            });
    }

    Ok(())
}
