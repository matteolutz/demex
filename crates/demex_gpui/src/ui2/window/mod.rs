use gpui::App;

pub mod outputs;
pub mod settings;

pub(super) fn init(cx: &mut App) {
    settings::init(cx);
    outputs::init(cx);
}
