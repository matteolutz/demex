use gpui::App;

pub mod outputs;
pub mod set_property;
pub mod settings;

pub(super) fn init(cx: &mut App) {
    settings::init(cx);
    outputs::init(cx);
    set_property::init(cx);
}
