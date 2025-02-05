pub mod dmx;
pub mod fixture;
pub mod lexer;
pub mod parser;
pub mod show;
pub mod ui;
pub mod utils;

use std::{
    sync::Arc,
    thread,
    time::{self, Duration},
};

use dmx::output::debug_dummy::{DebugDummyOutput, DebugDummyOutputVerbosity};
use fixture::{handler::FixtureHandler, patch::Patch};
use parking_lot::RwLock;
use show::DemexShow;
use ui::{DemexUiApp, DemexUiStats};

const TEST_SHOW_FILE: &str = "test_data/show.json";
const TEST_PATCH_FILE: &str = "test_data/patch.json";
const TEST_FUPS: f64 = 1000.0;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let show: DemexShow =
        serde_json::from_reader(std::fs::File::open(TEST_SHOW_FILE).unwrap()).unwrap();

    let ph = show.preset_handler;

    let patch: Patch =
        serde_json::from_reader(std::fs::File::open(TEST_PATCH_FILE).unwrap()).unwrap();

    let fixture_handler = Arc::new(RwLock::new(
        FixtureHandler::new(
            vec![
                Box::new(DebugDummyOutput::new(DebugDummyOutputVerbosity::Silent)),
                /*Box::new(
                    DMXSerialOutput::new("/dev/tty.usbserial-A10KPDBZ")
                        .expect("this shouldn't happen"),
                ),*/
            ],
            patch.clone().into(),
        )
        .unwrap(),
    ));

    let preset_handler = Arc::new(RwLock::new(ph));

    let stats = Arc::new(RwLock::new(DemexUiStats::default()));

    let ui_app_state = DemexUiApp::new(
        fixture_handler.clone(),
        preset_handler.clone(),
        patch,
        stats.clone(),
    );

    thread::spawn(move || {
        let mut last_update = time::Instant::now();
        let sleep_duration = Duration::from_secs_f64(1.0 / TEST_FUPS);

        loop {
            thread::sleep(sleep_duration);

            let elapsed = last_update.elapsed();

            let delta_time = elapsed.as_secs_f64();

            stats.write().fixed_update(delta_time);

            let mut fixture_handler = fixture_handler.write();
            let mut preset_handler = preset_handler.write();

            preset_handler.update_sequence_runtimes(delta_time, &mut fixture_handler);

            preset_handler.update_faders(delta_time);

            let _ = fixture_handler.update(&preset_handler, delta_time);

            last_update = time::Instant::now();
        }
    });

    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default().with_maximized(true),
        ..Default::default()
    };

    eframe::run_native("demex", options, Box::new(|_| Ok(Box::new(ui_app_state))))?;

    Ok(())
}
