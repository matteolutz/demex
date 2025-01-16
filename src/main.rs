pub mod dmx;
pub mod fixture;
pub mod lexer;
pub mod parser;
pub mod show;
pub mod ui;
pub mod utils;

use ui::DemexUiApp;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default().with_maximized(true),
        ..Default::default()
    };

    eframe::run_native(
        "demex",
        options,
        Box::new(|_| Ok(Box::<DemexUiApp>::default())),
    )?;

    Ok(())
}
