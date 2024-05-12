use ui::UIApp;

pub mod dmx;
pub mod fixture;
pub mod lexer;
pub mod parser;
pub mod ui;

// const SERIAL_PORT: &str = "/dev/tty.usbserial-A10KPDBZ";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default().with_inner_size([1120.0, 720.0]),
        ..Default::default()
    };
    eframe::run_native("demex", options, Box::new(|_| Box::<UIApp>::default()))?;

    Ok(())
}
