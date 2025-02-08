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
use ui::{stats::DemexUiStats, DemexUiApp};
use utils::deadlock::start_deadlock_checking_thread;

const TEST_SHOW_FILE: &str = "test_data/show.json";
const TEST_PATCH_FILE: &str = "test_data/patch.json";

const TEST_FUPS: f64 = 200.0;

const DEADLOCK_TEST: bool = true;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if DEADLOCK_TEST {
        start_deadlock_checking_thread();
    }

    let show: DemexShow =
        serde_json::from_reader(std::fs::File::open(TEST_SHOW_FILE).unwrap()).unwrap();

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

    let preset_handler = Arc::new(RwLock::new(show.preset_handler));
    let updatable_handler = Arc::new(RwLock::new(show.updatable_handler));

    let stats = Arc::new(RwLock::new(DemexUiStats::default()));

    let ui_app_state = DemexUiApp::new(
        fixture_handler.clone(),
        preset_handler.clone(),
        updatable_handler.clone(),
        patch,
        stats.clone(),
        |show: DemexShow| {
            serde_json::to_writer(std::fs::File::create(TEST_SHOW_FILE).unwrap(), &show)?;
            Ok(())
        },
    );

    thread::spawn(move || {
        let mut last_update = time::Instant::now();
        let sleep_duration = Duration::from_secs_f64(1.0 / TEST_FUPS);

        loop {
            let elapsed = last_update.elapsed();

            let delta_time = elapsed.as_secs_f64();

            stats.write().fixed_update(delta_time);

            {
                let preset_handler = preset_handler.read();
                let mut fixture_handler = fixture_handler.write();

                {
                    let mut updatable_handler = updatable_handler.write();

                    updatable_handler.update_faders(delta_time, &preset_handler);
                    updatable_handler.update_executors(
                        delta_time,
                        &mut fixture_handler,
                        &preset_handler,
                    );
                }

                let _ =
                    fixture_handler.update(&preset_handler, &updatable_handler.read(), delta_time);
            }

            last_update = time::Instant::now();

            thread::sleep(sleep_duration);
        }
    });

    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default().with_maximized(true),
        ..Default::default()
    };

    eframe::run_native("demex", options, Box::new(|_| Ok(Box::new(ui_app_state))))?;

    Ok(())
}
