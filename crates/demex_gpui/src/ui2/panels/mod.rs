use gpui::App;

pub mod command;
pub mod fixture_list;
pub mod fixture_selection;
pub mod performance;
pub mod pool;

pub fn init(cx: &mut App) {
    register_panels(cx);
}

fn register_panels(cx: &mut App) {
    command::register(cx);
    fixture_selection::register(cx);
    performance::register(cx);
    fixture_list::register(cx);
    pool::register(cx);
}
