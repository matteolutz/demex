#![warn(unused_extern_crates)]

pub mod dmx;
pub mod fixture;
pub mod input;
pub mod lexer;
pub mod parser;
pub mod show;
pub mod storage;
pub mod ui;
pub mod utils;

use std::{path::PathBuf, sync::Arc, time};

use egui::{Style, Visuals};
use fixture::{
    channel2::feature::feature_group::FeatureGroup, gdtf::GdtfFixture, handler::FixtureHandler,
};
use gdtf::GdtfFile;
use input::{device::DemexInputDeviceConfig, DemexInputDeviceHandler};
use itertools::Itertools;
use parking_lot::RwLock;
use rfd::FileDialog;
use show::DemexShow;
use ui::{error::DemexUiError, theme::DemexUiTheme, utils::icon::load_icon, DemexUiApp};
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
    #[arg(short, long)]
    show: Option<PathBuf>,

    /// Run an additional thread to periodically check for RwLock deadlocks
    #[arg(long)]
    deadlock_test: bool,
}

const TEST_MAX_FUPS: f64 = 60.0;
const TEST_MAX_DMX_FPS: f64 = 30.0;
const TEST_UI_FPS: f64 = 60.0;

const TEST_UI_THEME: DemexUiTheme = DemexUiTheme::Default;
const TEST_TOUCHSCREEN_FRIENDLY: bool = false;

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

    let show_file = args.show.clone();

    let fixture_files = std::fs::read_dir(storage::fixture_types(APP_ID))
        .unwrap()
        .flat_map(|file| {
            file.ok()
                .and_then(|f| std::fs::File::open(f.path()).ok())
                .and_then(|f| GdtfFile::new(f).ok())
        })
        .collect::<Vec<_>>();

    log::info!(
        "Found {} valid fixtures file(s), with {} fixture type(s)",
        fixture_files.len(),
        fixture_files
            .iter()
            .map(|f| f.description.fixture_types.len())
            .sum::<usize>()
    );
    log::debug!(
        "Valid fixture type(s): {}",
        fixture_files
            .iter()
            .flat_map(|f| &f.description.fixture_types)
            .map(|fixture_type| format!(
                "{} (man.: {}, id: {})",
                fixture_type.long_name, fixture_type.manufacturer, fixture_type.fixture_type_id
            ))
            .join(", ")
    );

    let mut gdtf_fixture = GdtfFixture::new(
        1,
        "Test Fixture".to_owned(),
        &fixture_files[1].description.fixture_types[0],
        "24CH".to_owned(),
        0,
        1,
    )
    .unwrap();
    /*gdtf_fixture
    .set_programmer_value(
        "Beam_Dimmer",
        FixtureChannelValue3::Discrete {
            channel_function_idx: 0,
            value: 1.0,
        },
    )
    .unwrap();*/
    log::info!("programmer values: {:?}", gdtf_fixture.programmer_values());

    let mut show: DemexShow = args
        .show
        .inspect(|show_path| log::info!("Loading show file: {:?}", show_path))
        .map(|show_path| serde_json::from_reader(std::fs::File::open(show_path).unwrap()).unwrap())
        .unwrap_or(DemexShow::default());

    *show.preset_handler.feature_groups_mut() = FeatureGroup::default_feature_groups();

    let fixture_handler = Arc::new(RwLock::new(
        FixtureHandler::new(show.patch, fixture_files).unwrap(),
    ));

    let preset_handler = Arc::new(RwLock::new(show.preset_handler));
    let updatable_handler = Arc::new(RwLock::new(show.updatable_handler));
    let timing_handler = Arc::new(RwLock::new(show.timing_handler));

    let stats = Arc::new(RwLock::new(DemexThreadStatsHandler::default()));

    let icon = Arc::new(load_icon());

    let ui_app_state = DemexUiApp::new(
        fixture_handler.clone(),
        preset_handler.clone(),
        updatable_handler.clone(),
        timing_handler.clone(),
        stats.clone(),
        show_file,
        |show: DemexShow, show_file: Option<&PathBuf>| {
            let save_file = if let Some(show_file) = show_file {
                show_file.clone()
            } else if let Some(save_file) = FileDialog::new()
                .add_filter("demex Show-File", &["json"])
                .save_file()
            {
                save_file
            } else {
                return Err(DemexUiError::RuntimeError("No save file selected".to_owned()).into());
            };

            serde_json::to_writer(std::fs::File::create(&save_file).unwrap(), &show)?;

            Ok(save_file)
        },
        TEST_UI_FPS,
        icon.clone(),
        DemexInputDeviceHandler::new(
            show.input_device_configs
                .into_iter()
                .map(DemexInputDeviceConfig::into)
                .collect::<Vec<_>>(),
        ),
        show.ui_config,
    );

    let fixture_handler_thread_a = fixture_handler.clone();
    let preset_handler_thread_a = preset_handler.clone();
    let updatable_handler_thread_a = updatable_handler.clone();
    let timing_handler_thread_a = timing_handler.clone();

    demex_update_thread(
        "demex-dmx-output".to_owned(),
        stats.clone(),
        TEST_MAX_DMX_FPS,
        move |delta_time, last_user_update| {
            let mut fixture_handler = fixture_handler_thread_a.write();
            let preset_handler = preset_handler_thread_a.read();
            let updatable_handler = updatable_handler_thread_a.read();
            let timing_handler = timing_handler_thread_a.read();

            if fixture_handler
                .update(
                    &preset_handler,
                    &updatable_handler,
                    &timing_handler,
                    delta_time,
                    last_user_update.elapsed().as_secs_f64() > 1.0,
                )
                .inspect_err(|err| log::error!("Failed to update fixture handler: {}", err))
                .is_ok_and(|res| res > 0)
            {
                *last_user_update = time::Instant::now();
            }
        },
    );

    demex_update_thread(
        "demex-update".to_owned(),
        stats.clone(),
        TEST_MAX_FUPS,
        move |delta_time, _| {
            let mut fixture_handler = fixture_handler.write();
            let preset_handler = preset_handler.read();
            let mut updatable_handler = updatable_handler.write();

            updatable_handler.update_faders(delta_time, &preset_handler);
            updatable_handler.update_executors(delta_time, &mut fixture_handler, &preset_handler);
        },
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

            if TEST_TOUCHSCREEN_FRIENDLY {
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

    Ok(())
}
