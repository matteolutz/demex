use std::sync::Arc;

use gpui::{Context, Entity, Render, Styled, Subscription, div};
use gpui::{prelude::*, px};
use gpui_component::dock::{DockArea, DockItem, DockPlacement};
use gpui_component::input::{Input, InputEvent, InputState};
use gpui_component::v_flex;

use crate::engine::DemexEngineHandler;
use crate::ui2::pane::panels::command::CommandPanel;
use crate::ui2::pane::panels::fixture_list::FixtureListPanel;
// use crate::ui2::pane::layout::LayoutViewPane;

// pub mod layout;
mod panels;

pub struct MainPane {
    // layout_pane: Entity<LayoutViewPane>,
    command_input_state: Entity<InputState>,
    dock_area: Entity<DockArea>,

    _subscriptions: Vec<Subscription>,
}

impl MainPane {
    pub fn new(window: &mut gpui::Window, cx: &mut Context<Self>) -> Self {
        let command_input_state = cx.new(|cx| InputState::new(window, cx).placeholder("Command"));
        let dock_area = cx.new(|cx| {
            let mut da = DockArea::new("demex-main-dock", None, window, cx);

            da.add_panel(
                Arc::new(FixtureListPanel::new("The first one", cx)),
                DockPlacement::Center,
                None,
                window,
                cx,
            );

            da.add_panel(
                Arc::new(FixtureListPanel::new("The second one", cx)),
                DockPlacement::Center,
                None,
                window,
                cx,
            );

            da.add_panel(
                Arc::new(FixtureListPanel::new("The third one", cx)),
                DockPlacement::Center,
                None,
                window,
                cx,
            );

            da.set_bottom_dock(
                DockItem::panel(Arc::new(CommandPanel::new(cx))),
                Some(5.0.into()),
                true,
                window,
                cx,
            );

            da
        });

        let subs = vec![cx.subscribe_in(
            &command_input_state,
            window,
            |_, input, event: &InputEvent, window, cx| match event {
                InputEvent::PressEnter { .. } => {
                    let command = input.read(cx).value();

                    if let Err(err) = DemexEngineHandler::engine(cx).exec_command(&command) {
                        log::warn!("Failed to run command \"{}\": {}", command, err);
                    }

                    input.update(cx, |input, cx| {
                        input.set_value("", window, cx);
                        cx.notify();
                    });
                }
                _ => {}
            },
        )];

        Self {
            // layout_pane: cx.new(|cx| LayoutViewPane::new(window, cx)),
            command_input_state,
            dock_area,
            _subscriptions: subs,
        }
    }
}

impl Render for MainPane {
    fn render(
        &mut self,
        window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        v_flex()
            .size_full()
            .child(div().w_full().h(px(20.0)).child("demex"))
            .child(
                div()
                    .w_full()
                    .flex_1()
                    // .child(self.layout_pane.clone()),
                    .child(self.dock_area.clone()),
            )
            .child(
                div()
                    .w_full()
                    .h(px(50.0))
                    .flex()
                    .items_center()
                    .p_2()
                    .child(
                        div()
                            .p_1()
                            .w_full()
                            .border_1()
                            .child(Input::new(&self.command_input_state)),
                    ),
            )
    }
}
