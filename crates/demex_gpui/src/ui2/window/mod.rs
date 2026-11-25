use gpui::App;

pub mod add_fixture;
pub mod add_pool_window;
pub mod edit_cue_trigger;
pub mod edit_keyframe_effect;
pub mod outputs;
pub mod patch;
pub mod set_attribute;
pub mod set_property;
pub mod settings;

pub(super) fn init(cx: &mut App) {
    settings::init(cx);
    outputs::init(cx);
    set_property::init(cx);
    edit_cue_trigger::init(cx);
    edit_keyframe_effect::init(cx);
    set_attribute::init(cx);
    add_pool_window::init(cx);
    add_fixture::init(cx);
    patch::init(cx);
}
