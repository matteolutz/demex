#![warn(unused_extern_crates)]

pub mod app;
pub mod engine;
pub mod storage;
pub mod ui2;
pub mod utils;

use std::path::PathBuf;

use gdtf::GdtfFile;
use itertools::Itertools;

use demex_core::utils::deadlock::start_deadlock_checking_thread;

use clap::Parser;

use crate::{
    app::{DemexApp, DemexAppArgs},
    storage::read_or_create_dir,
};

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
    ui_theme: Option<String>,

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

    /// Disable autosave in the UI.
    #[arg(long)]
    no_autosave: bool,

    /// Enable the controller mode, which allows the application to act as a controller for headless nodes.
    #[arg(long, default_value = "false", conflicts_with = "headless")]
    controller: bool,

    #[cfg(debug_assertions)]
    #[arg(long, default_value = "false")]
    backtrace: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(debug_assertions)]
    {
        if std::env::var("RUST_LOG").is_err() {
            unsafe { std::env::set_var("RUST_LOG", "debug") }
        }
    }

    env_logger::init();
    log::info!("Starting demex");

    let args = Args::parse();

    #[cfg(debug_assertions)]
    {
        let backtrace = if args.backtrace { "1" } else { "0" };
        log::info!("Running with RUST_BACKTRACE={}", backtrace);
        unsafe { std::env::set_var("RUST_BACKTRACE", backtrace) }
    }

    if args.deadlock_test {
        start_deadlock_checking_thread();
    }

    let fixture_files = read_or_create_dir(storage::fixture_types())
        .expect("Failed to read or create fixture types directory")
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
        storage::fixture_types().display()
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

        DemexApp::default().run(DemexAppArgs {
            touchscreen_mode: args.touchscreen_mode,
            showfile_path: args.show,
            fixture_types,
            theme: args.ui_theme,
            additional_viewports: args.additional_viewports.unwrap_or(0),
            disable_autosave: args.no_autosave,
        });
    }

    Ok(())
}
