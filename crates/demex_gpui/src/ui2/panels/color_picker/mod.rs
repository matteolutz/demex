use gpui::{
    App, AppContext, Context, Entity, EventEmitter, FocusHandle, Focusable, ParentElement, Render,
    Rgba, Styled, Subscription, Window,
};
use gpui_component::{
    Sizable,
    color_picker::{ColorPicker, ColorPickerState},
    dock::PanelEvent,
    h_flex,
};

use crate::{
    engine::state::DemexUiState,
    ui2::{
        panels::DemexPanel,
        utils::{additive_attribute_to_rgba, subtractive_attribute_to_rgba_factor},
    },
};

pub struct ColorPickerPanel {
    focus_handle: FocusHandle,

    color_picker_state: Entity<ColorPickerState>,

    _subscriptions: Vec<Subscription>,
}

impl ColorPickerPanel {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let fixture_color = Self::get_color_from_fixture_values(cx);
        let color_picker_state = cx.new(|cx| {
            ColorPickerState::new(window, cx).default_value(fixture_color.unwrap_or(Rgba {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.0,
            }))
        });

        let _subscriptions = vec![
            cx.observe_in(
                &DemexUiState::fixture_selection(cx),
                window,
                |this, _, window, cx| {
                    this.update_color_from_fixture_values(window, cx);
                },
            ),
            cx.observe_in(
                &DemexUiState::fixture_values(cx),
                window,
                |this, _, window, cx| {
                    this.update_color_from_fixture_values(window, cx);
                },
            ),
        ];

        Self {
            focus_handle: cx.focus_handle(),

            color_picker_state,
            _subscriptions,
        }
    }

    fn update_color_from_fixture_values(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let color = Self::get_color_from_fixture_values(cx).unwrap_or(Rgba {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        });

        self.color_picker_state.update(cx, move |cp_state, cx| {
            cp_state.set_value(color, window, cx);
        });
        cx.notify();
    }

    fn get_color_from_fixture_values(cx: &App) -> Option<Rgba> {
        let master_fixture = DemexUiState::fixture_selection(cx)
            .read(cx)
            .as_ref()?
            .selection()
            .master_fixture()?;

        let master_fixture_values = DemexUiState::fixture_values(cx)
            .read(cx)
            .get(master_fixture)?;

        let master_fixture_patch = DemexUiState::patch(cx)
            .read(cx)
            .fixture(master_fixture)
            .unwrap();

        // additve color mixing
        let mut additive_result = None;
        for (attr, value) in master_fixture_values {
            let Some(f_value) = master_fixture_patch
                .channel_function(attr)
                .and_then(|cf| value.try_as_discrete().map(|value| value.to_clamped(cf)))
            else {
                continue;
            };

            let Some(attr_result) = additive_attribute_to_rgba(*attr, f_value.as_f32()) else {
                continue;
            };

            let additive_result = additive_result.get_or_insert(Rgba {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.0,
            });

            additive_result.r += attr_result.r;
            additive_result.g += attr_result.g;
            additive_result.b += attr_result.b;
        }

        // subtractive color mixing
        let mut subtractive_result = None;
        for (attr, value) in master_fixture_values {
            let Some(f_value) = master_fixture_patch
                .channel_function(attr)
                .and_then(|cf| value.try_as_discrete().map(|value| value.to_clamped(cf)))
            else {
                continue;
            };

            let Some(attr_result) = subtractive_attribute_to_rgba_factor(*attr, f_value.as_f32())
            else {
                continue;
            };

            let subtractive_result = subtractive_result.get_or_insert(Rgba {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 0.0,
            });

            subtractive_result.r *= attr_result.r;
            subtractive_result.g *= attr_result.g;
            subtractive_result.b *= attr_result.b;
        }

        if additive_result.is_none() && subtractive_result.is_none() {
            return None;
        }

        let mut color = additive_result.unwrap_or(Rgba {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        });
        if let Some(subtractive_result) = subtractive_result {
            color.r *= subtractive_result.r;
            color.g *= subtractive_result.g;
            color.b *= subtractive_result.b;
        }

        Some(color)
    }
}

impl EventEmitter<PanelEvent> for ColorPickerPanel {}
impl Focusable for ColorPickerPanel {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl DemexPanel for ColorPickerPanel {
    fn panel_type() -> super::DockWindowPanelType {
        super::DockWindowPanelType::ColorPicker
    }

    fn deserialize(
        _dock_area: gpui::WeakEntity<gpui_component::dock::DockArea>,
        _panel_state: &gpui_component::dock::PanelState,
        _panel_info: &gpui_component::dock::PanelInfo,
        window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> Self {
        Self::new(window, cx)
    }
}

impl Render for ColorPickerPanel {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        h_flex()
            .p_2()
            .justify_center()
            .size_full()
            .child(ColorPicker::new(&self.color_picker_state).large())
    }
}
