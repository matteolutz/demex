use std::time::{self};

use demex_core::{
    color::color_space::RgbValue,
    command::parser::nodes::{
        action::{
            Action,
            functions::set_function::{SetFeatureValue, SetFeatureValueArgs},
        },
        fixture_selector::FixtureSelector,
    },
};
use gpui::{
    App, AppContext, Context, Entity, EventEmitter, FocusHandle, Focusable, ParentElement, Render,
    Rgba, Styled, Subscription, Task, Window,
};
use gpui_component::{
    Sizable,
    checkbox::Checkbox,
    color_picker::{ColorPicker, ColorPickerEvent, ColorPickerState},
    dock::PanelEvent,
    v_flex,
};

use crate::{
    engine::{DemexEngineHandler, state::DemexUiState},
    ui2::{
        panels::DemexPanel,
        utils::{additive_attribute_to_rgba, subtractive_attribute_to_rgba_factor},
    },
};

const COLOR_PICKER_DEBOUNCE_TIME: time::Duration = time::Duration::from_millis(500);

pub struct ColorPickerPanel {
    focus_handle: FocusHandle,

    color_picker_state: Entity<ColorPickerState>,

    /// When the color picker was last touched by the user.
    last_color_picker_change: Option<time::Instant>,
    /// When the color picker should be updated from the fixtures values.
    next_update_from_fixtures: Option<Task<()>>,

    use_white: bool,

    _subscriptions: Vec<Subscription>,
}

impl ColorPickerPanel {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let use_white = false;

        let fixture_color = Self::get_color_from_fixture_values(use_white, cx);
        let color_picker_state = cx.new(|cx| {
            ColorPickerState::new(window, cx).default_value(fixture_color.unwrap_or(Rgba {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
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
            cx.subscribe(&color_picker_state, |this, _, evt, cx| match evt {
                ColorPickerEvent::Change(color) => {
                    if let Some(color) = color.map(|c| c.to_rgb()) {
                        this.last_color_picker_change = Some(time::Instant::now());

                        DemexEngineHandler::engine(cx).exec_ui(Action::SetFeatureValue(
                            SetFeatureValueArgs {
                                fixture_selector: FixtureSelector::current_fixtures_selected(),
                                feature: SetFeatureValue::Rgb {
                                    value: RgbValue::srgb(color.r, color.g, color.b),
                                    use_white: this.use_white,
                                },
                            },
                        ));
                    }
                }
            }),
        ];

        Self {
            focus_handle: cx.focus_handle(),

            color_picker_state,

            last_color_picker_change: None,
            next_update_from_fixtures: None,

            use_white,

            _subscriptions,
        }
    }

    fn update_color_from_fixture_values(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // drop any pending update task, we'll replace it now
        self.next_update_from_fixtures.take();

        if let Some(elapsed_since_last_change) = self
            .last_color_picker_change
            .map(|last_change| last_change.elapsed())
        {
            if elapsed_since_last_change < COLOR_PICKER_DEBOUNCE_TIME {
                // we need to wait for the debounce period to elapse before updating
                // so we will need to schedule an update task
                self.next_update_from_fixtures =
                    Some(cx.spawn_in(window, async move |this, cx| {
                        // wait for the debounce period to elapse before updating
                        cx.background_executor()
                            .timer(COLOR_PICKER_DEBOUNCE_TIME - elapsed_since_last_change)
                            .await;

                        let _ = this.update_in(cx, |this, window, cx| {
                            this.update_color_from_fixture_values(window, cx)
                        });
                    }));
                return;
            }
        }

        let color = Self::get_color_from_fixture_values(self.use_white, cx).unwrap_or(Rgba {
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

    fn get_color_from_fixture_values(use_white: bool, cx: &App) -> Option<Rgba> {
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

            let Some(attr_result) = additive_attribute_to_rgba(*attr, f_value.as_f32(), use_white)
            else {
                continue;
            };

            let additive_result = additive_result.get_or_insert(Rgba {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
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
                a: 1.0,
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
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        v_flex()
            .p_2()
            .gap_4()
            .justify_center()
            .items_center()
            .size_full()
            .child(ColorPicker::new(&self.color_picker_state).large())
            .child(
                Checkbox::new("use-white")
                    .label("Use White channels")
                    .checked(self.use_white)
                    .on_click(cx.listener(|this, &checked, _, cx| {
                        this.use_white = checked;
                        cx.notify();
                    })),
            )
    }
}
