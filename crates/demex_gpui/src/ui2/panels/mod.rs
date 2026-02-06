use gpui::{App, Context, Window};
use gpui_component::{IconName, button::Button, dock::Panel};
use strum::IntoEnumIterator;

use crate::ui2::wm::WindowManager;

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

#[derive(Debug, Copy, Clone, strum::EnumIter, strum::Display)]
pub enum DockWindowPanelType {
    AttributeEditor,
    FixtureList,
    FixtureSelection,
    LayoutView,
    Multipool,
    SequenceEditor,
}

fn toolbar_buttons<P: Panel>(_this: &P, _window: &mut Window, _cx: &mut Context<P>) -> Vec<Button> {
    vec![
        Button::new("add-panel")
            .icon(IconName::Plus)
            .on_click(|evt, window, cx| {
                let window_handle = window.window_handle();
                let pos = evt.position();

                cx.defer(move |cx| {
                    let _ =
                        WindowManager::update_dock_window_handle(window_handle, cx, |dw, _, cx| {
                            dw.context_layer().update(cx, |context, cx| {
                                context.context_menu(
                                    pos,
                                    DockWindowPanelType::iter().map(|panel_type| {
                                        (
                                            panel_type.to_string(),
                                            move |window: &mut Window, cx: &mut App| {
                                                let window_handle = window.window_handle();
                                                cx.defer(move |cx| {
                                                    let _ =
                                                        WindowManager::update_dock_window_handle(
                                                            window_handle,
                                                            cx,
                                                            |dw, window, cx| {
                                                                dw.add_panel(panel_type, window, cx)
                                                            },
                                                        );
                                                });
                                            },
                                        )
                                    }),
                                    cx,
                                );
                            });
                        });
                });
            }),
    ]
}

pub fn init(cx: &mut App) {
    register_panels(cx);

    command::init(cx);
    multipool::init(cx);
}

fn register_panels(cx: &mut App) {
    command::register(cx);
    fixture_selection::register(cx);
    performance::register(cx);
    fixture_list::register(cx);
    // pool::register(cx);
    layout_view::register(cx);
    sequence_editor::register(cx);
    attribute_editor::register(cx);
    multipool::register(cx);
}
