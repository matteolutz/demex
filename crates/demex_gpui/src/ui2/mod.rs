use gpui::App;

pub mod assets;
pub mod components;
pub mod config;
pub mod ext;
pub mod pane;
pub mod titlebar;
pub mod utils;
pub mod window;

pub fn init(cx: &mut App) -> gpui::Result<()> {
    assets::init(cx)?;
    Ok(())
}
