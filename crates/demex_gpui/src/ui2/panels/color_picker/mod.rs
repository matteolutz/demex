use std::{
    sync::Arc,
    time::{self},
};

use demex_core::{
    color::color_space::{RgbColorSpace, RgbValue},
    command::parser::nodes::{
        action::{
            Action,
            functions::set_function::{SetFeatureValue, SetFeatureValueArgs},
        },
        fixture_selector::FixtureSelector,
    },
};
use gpui::{
    App, AppContext, Bounds, ClickEvent, Context, Corners, Edges, Entity, EventEmitter,
    FocusHandle, Focusable, Image, InteractiveElement, ParentElement, PathBuilder, Pixels, Render,
    Rgba, StatefulInteractiveElement, Styled, StyledImage, Subscription, Task, Window, canvas, div,
    img, point, px, quad, size, transparent_black,
};
use gpui_component::{
    IndexPath, Sizable, black,
    checkbox::Checkbox,
    color_picker::{ColorPicker, ColorPickerEvent, ColorPickerState},
    dock::PanelEvent,
    h_flex,
    select::{Select, SelectState},
    v_flex,
};
use itertools::Itertools;
use strum::IntoEnumIterator;

use crate::{
    engine::{DemexEngineHandler, state::DemexUiState},
    ui2::{
        ext::{BoundsExt, GpuiContextExtension},
        panels::{DemexPanel, color_picker::color_space::ColorSpaceSelectItem},
        utils::{additive_attribute_to_rgba, bounds, subtractive_attribute_to_rgba_factor},
    },
};

mod color_space;

const CIE: &[u8] = include_bytes!(
    "../../../../../../assets/cie/CIE_1931_Chromaticity_Diagram_CIE_1931_2_Degree_Standard_Observer.png"
);

const COLOR_PICKER_DEBOUNCE_TIME: time::Duration = time::Duration::from_millis(500);

pub struct ColorPickerPanel {
    focus_handle: FocusHandle,

    color_picker_state: Entity<ColorPickerState>,

    /// When the color picker was last touched by the user.
    last_color_picker_change: Option<time::Instant>,
    /// When the color picker should be updated from the fixtures values.
    next_update_from_fixtures: Option<Task<()>>,

    /// Whether to use the additive white channels of fixtures
    /// when calculating or setting the color
    use_white: bool,

    cie_image: Arc<Image>,
    cie_bounds: Entity<Option<Bounds<Pixels>>>,

    color_space_select_state: Entity<SelectState<Vec<ColorSpaceSelectItem>>>,

    _subscriptions: Vec<Subscription>,
}

impl ColorPickerPanel {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let use_white = false;

        let color_space_select_state = cx.new(|cx| {
            SelectState::new(
                RgbColorSpace::iter()
                    .map_into()
                    .collect::<Vec<ColorSpaceSelectItem>>(),
                Some(IndexPath::new(0)),
                window,
                cx,
            )
        });

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
                    if let Some((color, &color_space)) = color
                        .map(|c| c.to_rgb())
                        .zip(this.color_space_select_state.read(cx).selected_value())
                    {
                        this.last_color_picker_change = Some(time::Instant::now());
                        this.set_color(RgbValue::new(color.r, color.g, color.b, color_space), cx);
                    }
                }
            }),
            cx.observe_and_notify(&color_space_select_state),
        ];

        Self {
            focus_handle: cx.focus_handle(),

            color_picker_state,

            last_color_picker_change: None,
            next_update_from_fixtures: None,

            use_white,

            cie_image: Arc::new(Image::from_bytes(gpui::ImageFormat::Png, CIE.to_vec())),
            cie_bounds: cx.new(|_| None),

            color_space_select_state,

            _subscriptions,
        }
    }

    fn set_color(&self, color: RgbValue, cx: &mut App) {
        DemexEngineHandler::engine(cx).exec_ui(Action::SetFeatureValue(SetFeatureValueArgs {
            fixture_selector: FixtureSelector::current_fixtures_selected(),
            feature: SetFeatureValue::Rgb {
                value: color,
                use_white: self.use_white,
            },
        }));
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
            .p_4()
            .gap_2()
            .items_center()
            .size_full()
            .child(
                h_flex()
                    .gap_4()
                    .w_full()
                    .justify_center()
                    .items_center()
                    .child(Select::new(&self.color_space_select_state))
                    .child(ColorPicker::new(&self.color_picker_state).large()),
            )
            .child(
                h_flex().w_full().child(
                    Checkbox::new("use-white")
                        .label("Use White channels")
                        .checked(self.use_white)
                        .on_click(cx.listener(|this, &checked, _, cx| {
                            this.use_white = checked;
                            cx.notify();
                        })),
                ),
            )
            .child(
                div()
                    .id("cie")
                    .relative()
                    .w_64()
                    .h_64()
                    .child(
                        img(self.cie_image.clone())
                            .absolute()
                            .top_0()
                            .left_0()
                            .size_full()
                            .object_fit(gpui::ObjectFit::Fill),
                    )
                    .child(
                        bounds(&self.cie_bounds)
                            .absolute()
                            .top_0()
                            .left_0()
                            .size_full(),
                    )
                    .child(
                        canvas(
                            |_, _, _| {},
                            cx.draw_canvas(|this, bounds, _, window, cx| {
                                // draw color space
                                let Some(&color_space) =
                                    this.color_space_select_state.read(cx).selected_value()
                                else {
                                    return;
                                };

                                let mut path_builder = PathBuilder::stroke(px(2.0));
                                for color in color_space.gammut() {
                                    let (x, y) = color.to_xy();
                                    let Some(absolute_point) =
                                        bounds.map_point(&point(px(x), px(1.0 - y)))
                                    else {
                                        // this shouldn't happen, but you never know
                                        return;
                                    };

                                    // the first call to line_to will result in a move_to like call,
                                    // so we don't need to check idx
                                    path_builder.line_to(absolute_point);
                                }
                                path_builder.close(); // make sure, we connect the last point to the first one

                                window.paint_path(path_builder.build().unwrap(), black());

                                // draw current color
                                let Some(current_rgb_color) = this
                                    .color_picker_state
                                    .read(cx)
                                    .value()
                                    .map(|val| val.to_rgb())
                                else {
                                    return;
                                };

                                let (x, y) = RgbValue::new(
                                    current_rgb_color.r,
                                    current_rgb_color.g,
                                    current_rgb_color.b,
                                    color_space,
                                )
                                .to_xy();

                                let Some(absolute_point) =
                                    bounds.map_point(&point(px(x), px(1.0 - y)))
                                else {
                                    // this shouldn't happen, but you never know
                                    return;
                                };

                                window.paint_quad(quad(
                                    Bounds::centered_at(absolute_point, size(px(10.0), px(10.0))),
                                    Corners::all(px(5.0)),
                                    Rgba {
                                        r: 1.0 - current_rgb_color.r,
                                        g: 1.0 - current_rgb_color.g,
                                        b: 1.0 - current_rgb_color.b,
                                        a: 1.0,
                                    },
                                    Edges::all(px(0.0)),
                                    transparent_black(),
                                    gpui::BorderStyle::Solid,
                                ));
                            }),
                        )
                        .absolute()
                        .top_0()
                        .left_0()
                        .size_full(),
                    )
                    .on_click(cx.listener(|this, evt: &ClickEvent, _, cx| {
                        let Some(cie_bounds) = this.cie_bounds.read(cx) else {
                            return;
                        };

                        let Some(cie_pos) = cie_bounds.unmap_point(&evt.position()).map(|mut p| {
                            p.y = px(1.0) - p.y; // invert the y axis
                            p
                        }) else {
                            return;
                        };

                        log::debug!("cie xy: ({}, {})", cie_pos.x.as_f32(), cie_pos.y.as_f32());

                        let Some(&color_space) =
                            this.color_space_select_state.read(cx).selected_value()
                        else {
                            return;
                        };

                        let rgb_value = RgbValue::from_xyy(
                            cie_pos.x.as_f32(),
                            cie_pos.y.as_f32(),
                            1.0,
                            color_space,
                        );

                        this.set_color(rgb_value, cx);
                    })),
            )
    }
}
