use gpui::App;
use gpui::Bounds;
use gpui::DefiniteLength;
use gpui::Entity;
use gpui::MouseButton;
use gpui::MouseMoveEvent;
use gpui::Pixels;
use gpui::Render;
use gpui::Window;
use gpui::div;
use gpui::prelude::*;
use gpui::px;

use crate::container::container;
use crate::theme::ActiveTheme;
use crate::utils::bounds_updater;

const THUMB_HEIGHT: Pixels = px(20.0);

pub struct Fader {
    bounds: Entity<Option<Bounds<Pixels>>>,
    fader_value: Entity<f32>,
}

impl Fader {
    pub fn new(fader_value: Entity<f32>, _window: &mut Window, cx: &mut App) -> Self {
        Self {
            bounds: cx.new(|_| None),
            fader_value,
        }
    }

    pub fn fader_value(&self) -> Entity<f32> {
        self.fader_value.clone()
    }
}

impl Render for Fader {
    fn render(&mut self, window: &mut gpui::Window, cx: &mut Context<Self>) -> impl IntoElement {
        let fader_value = *self.fader_value.read(cx);
        let fader_pos = self
            .bounds
            .read(cx)
            .map(|bounds| fader_value * ((bounds.size.height - THUMB_HEIGHT) / bounds.size.height))
            .unwrap_or(0.0);

        div()
            .bg(cx.theme().input_secondary)
            .h_full()
            .w_20()
            .border_1()
            .border_color(cx.theme().border)
            .relative()
            .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, _, cx| {
                if event.pressed_button != Some(MouseButton::Left) {
                    return;
                }

                let Some(bounds) = this.bounds.read(cx).clone() else {
                    return;
                };

                let Some(localized_mouse_pos) = bounds.localize(&event.position) else {
                    return;
                };

                let normalized_mouse_y = localized_mouse_pos.y / bounds.size.height;

                this.fader_value.update(cx, |fader_value, cx| {
                    *fader_value = ((1.0 - normalized_mouse_y)
                        * (bounds.size.height / (bounds.size.height - THUMB_HEIGHT)))
                        .clamp(0.0, 1.0);
                    cx.notify();
                });
            }))
            .child(bounds_updater(
                self.bounds.clone(),
                |bounds, updated_bounds, _| {
                    *bounds = Some(updated_bounds);
                },
            ))
            .child(
                container(window, cx)
                    .absolute()
                    .w_full()
                    .h(THUMB_HEIGHT)
                    .bottom(DefiniteLength::Fraction(fader_pos))
                    .flex()
                    .justify_center()
                    .items_center()
                    .text_center()
                    .child(format!("{:.1}%", fader_value * 100.0)),
            )
            .child(
                div()
                    .absolute()
                    .w_full()
                    .bottom_0()
                    .h(DefiniteLength::Fraction(fader_pos))
                    .bg(cx.theme().accent),
            )
    }
}
