#![warn(unused_extern_crates)]

pub mod dmx;
pub mod fixture;
pub mod lexer;
pub mod parser;
pub mod show;
pub mod ui;
pub mod utils;

use std::{path::PathBuf, sync::Arc, time};

use egui::{Style, Visuals};
use fixture::handler::FixtureHandler;
use parking_lot::RwLock;
use show::DemexShow;
use ui::{utils::icon::load_icon, DemexUiApp};
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

const TEST_MAX_FUPS: f64 = 200.0;
const TEST_MAX_DMX_FPS: f64 = 30.0;
const TEST_UI_FPS: f64 = 60.0;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if args.deadlock_test {
        start_deadlock_checking_thread();
    }

    let show: DemexShow = args
        .show
        .map(|show_path| serde_json::from_reader(std::fs::File::open(show_path).unwrap()).unwrap())
        .unwrap_or(DemexShow::default());

    let fixture_handler = Arc::new(RwLock::new(FixtureHandler::new(show.patch).unwrap()));

    let preset_handler = Arc::new(RwLock::new(show.preset_handler));
    let updatable_handler = Arc::new(RwLock::new(show.updatable_handler));

    let stats = Arc::new(RwLock::new(DemexThreadStatsHandler::default()));

    let icon = Arc::new(load_icon());

    let ui_app_state = DemexUiApp::new(
        fixture_handler.clone(),
        preset_handler.clone(),
        updatable_handler.clone(),
        stats.clone(),
        |show: DemexShow| {
            serde_json::to_writer(std::fs::File::create("test_data/show.json").unwrap(), &show)?;
            Ok(())
        },
        TEST_UI_FPS,
        icon.clone(),
    );

    let fixture_handler_thread_a = fixture_handler.clone();
    let preset_handler_thread_a = preset_handler.clone();
    let updatable_handler_thread_a = updatable_handler.clone();

    demex_update_thread(
        "demex-dmx-output".to_owned(),
        stats.clone(),
        TEST_MAX_DMX_FPS,
        move |delta_time, last_user_update| {
            let mut fixture_handler = fixture_handler_thread_a.write();
            let preset_handler = preset_handler_thread_a.read();
            let updatable_handler = updatable_handler_thread_a.read();

            if fixture_handler
                .update(
                    &preset_handler,
                    &updatable_handler,
                    delta_time,
                    last_user_update.elapsed().as_secs_f64() > 1.0,
                )
                .unwrap()
                > 0
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
        "demex",
        options,
        Box::new(|creation_context| {
            let style = Style {
                visuals: Visuals::dark(),
                ..Style::default()
            };
            creation_context.egui_ctx.set_style(style);
            Ok(Box::new(ui_app_state))
        }),
    )?;

    Ok(())
}
