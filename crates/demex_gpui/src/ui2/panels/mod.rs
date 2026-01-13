use gpui::{App, Context, Window};
use gpui_component::{IconName, button::Button, dock::Panel};

pub mod attribute_editor;
pub mod command;
pub mod fixture_list;
pub mod fixture_selection;
pub mod layout_view;
pub mod multipool;
pub mod performance;
pub mod pool;
pub mod sequence_editor;

mod actions {
    gpui::actions!(panels, [AddFixtureSelection, AddFixtureList, AddLayoutView]);
}

fn toolbar_buttons<P: Panel>(_this: &P, _window: &mut Window, _cx: &mut Context<P>) -> Vec<Button> {
    vec![Button::new("add-panel").icon(IconName::Plus)]
}

pub fn init(cx: &mut App) {
    register_panels(cx);

    command::init(cx);
}

fn register_panels(cx: &mut App) {
    command::register(cx);
    fixture_selection::register(cx);
    performance::register(cx);
    fixture_list::register(cx);
    pool::register(cx);
    layout_view::register(cx);
    sequence_editor::register(cx);
    attribute_editor::register(cx);
    multipool::register(cx);
}
