use demex_core::{
    channel3::clamped_value::ClampedValue,
    command::parser::nodes::action::Action,
    engine::comm::KeyframeEffectRequest,
    keyframe_effect::{
        effect_preset::{
            KeyframeEffectPreset, PanTiltSingleOriginEffectPreset,
            PanTiltSingleOriginEffectPresetType,
        },
        effect_runtime::KeyframeEffectRuntime,
    },
    presets::preset::FixturePresetId,
};
use gpui::{
    AppContext, Bounds, ClickEvent, Context, Edges, Entity, PaintQuad, ParentElement, Render, Size,
    Styled, Subscription, Window, WindowBounds, div, prelude::FluentBuilder, px, size,
    transparent_black,
};
use gpui_component::{
    IndexPath,
    button::Button,
    h_flex,
    select::{Select, SelectItem, SelectState},
    slider::{Slider, SliderState},
    v_flex, white,
};
use itertools::Itertools;
use strum::IntoEnumIterator;

use crate::{
    engine::DemexEngineHandler,
    ui2::{
        components::pan_tilt_editor::{PanTiltEditor, PanTiltEditorState},
        ext::{BoundsExt, GpuiContextExtension},
        wm::edit_window::EditWindowDelegate,
    },
};

#[derive(Debug, Copy, Clone)]
struct PanTiltSingleOriginEffectPresetTypeSelectItem(PanTiltSingleOriginEffectPresetType);

impl From<PanTiltSingleOriginEffectPresetType> for PanTiltSingleOriginEffectPresetTypeSelectItem {
    fn from(value: PanTiltSingleOriginEffectPresetType) -> Self {
        Self(value)
    }
}

impl SelectItem for PanTiltSingleOriginEffectPresetTypeSelectItem {
    type Value = PanTiltSingleOriginEffectPresetType;

    fn title(&self) -> gpui::SharedString {
        self.0.to_string().into()
    }

    fn value(&self) -> &Self::Value {
        &self.0
    }
}

struct PresetPanTiltSingleOriginState {
    editor_state: Entity<PanTiltEditorState>,
    move_type_state: Entity<SelectState<Vec<PanTiltSingleOriginEffectPresetTypeSelectItem>>>,

    width_state: Entity<SliderState>,
    height_state: Entity<SliderState>,

    _subscriptions: Vec<Subscription>,
}

impl PresetPanTiltSingleOriginState {
    pub fn new(
        effect: KeyframeEffectRuntime,
        window: &mut Window,
        cx: &mut Context<PresetPanTiltSingleOriginWindow>,
    ) -> Self {
        let preset = effect.effect().preset();

        let editor_state = cx.new(|cx| {
            PanTiltEditorState::new(
                match preset {
                    Some(KeyframeEffectPreset::PanTiltSingleOrigin(preset)) => preset.origin,
                    _ => [0.5.into(), 0.5.into()],
                },
                cx,
            )
        });

        let move_type_state = cx.new(|cx| {
            SelectState::new(
                PanTiltSingleOriginEffectPresetType::iter()
                    .map_into()
                    .collect(),
                match preset {
                    Some(KeyframeEffectPreset::PanTiltSingleOrigin(preset)) => {
                        PanTiltSingleOriginEffectPresetType::iter()
                            .find_position(|t| *t == preset.move_type)
                            .map(|(idx, _)| IndexPath::new(idx))
                    }
                    None => None,
                },
                window,
                cx,
            )
        });

        let [width, height] = preset
            .and_then(|p| match p {
                KeyframeEffectPreset::PanTiltSingleOrigin(preset) => Some(preset.size),
                _ => None,
            })
            .unwrap_or([0.5.into(), 0.5.into()]);

        let width_state = cx.new(|_| {
            SliderState::new()
                .min(0.0)
                .max(1.0)
                .step(0.001)
                .default_value(width.as_f32())
        });
        let height_state = cx.new(|_| {
            SliderState::new()
                .min(0.0)
                .max(1.0)
                .step(0.001)
                .default_value(height.as_f32())
        });

        let _subscriptions = vec![cx.observe_and_notify(&editor_state)];

        Self {
            editor_state,
            move_type_state,
            width_state,
            height_state,

            _subscriptions,
        }
    }
}

pub struct PresetPanTiltSingleOriginWindow {
    preset_id: FixturePresetId,

    state: Option<PresetPanTiltSingleOriginState>,
}

impl PresetPanTiltSingleOriginWindow {
    pub fn new(preset_id: FixturePresetId, window: &mut Window, cx: &mut Context<Self>) -> Self {
        DemexEngineHandler::send_in_visual(
            window,
            cx,
            KeyframeEffectRequest { preset_id },
            |this, effect, window, cx| {
                let Some(effect) = effect else {
                    return;
                };

                this.state = Some(PresetPanTiltSingleOriginState::new(effect, window, cx));
                cx.notify();
            },
        );

        Self {
            preset_id,
            state: None,
        }
    }

    pub fn apply_preset(&mut self, _: &ClickEvent, _: &mut Window, cx: &mut Context<Self>) {
        let Some(state) = self.state.as_ref() else {
            return;
        };

        let Some(move_type) = state.move_type_state.read(cx).selected_value().copied() else {
            return;
        };

        DemexEngineHandler::engine(cx).exec_ui(Action::PresetApplyKeyframeEffectPreset(
            self.preset_id,
            KeyframeEffectPreset::PanTiltSingleOrigin(PanTiltSingleOriginEffectPreset {
                size: [
                    state.width_state.read(cx).value().start().into(),
                    state.height_state.read(cx).value().start().into(),
                ],
                origin: state.editor_state.read(cx).value(),
                move_type,
                rotation: 0.0,
            }),
        ));
        self.discard_and_close(cx);
    }
}

impl Render for PresetPanTiltSingleOriginWindow {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        v_flex()
            .p_4()
            .gap_4()
            .size_full()
            .when_some(self.state.as_ref(), |this, state| {
                let move_type = state.move_type_state.read(cx).selected_value().copied();
                let size: Size<ClampedValue> = size(
                    state.width_state.read(cx).value().start().into(),
                    state.height_state.read(cx).value().start().into(),
                );

                this.child(
                    h_flex()
                        .gap_4()
                        .size_full()
                        .child(
                            div()
                                .w_full()
                                .child(PanTiltEditor::new(&state.editor_state).paint(
                                    move |bounds, center_pos, window, _| {
                                        let Some(move_type) = move_type else {
                                            return;
                                        };

                                        let mapped_size = bounds.map_clamped_size(&size);

                                        match move_type {
                                            PanTiltSingleOriginEffectPresetType::Ellipse => {
                                                let x_radius = mapped_size.width / 2.0;
                                                let y_radius = mapped_size.height / 2.0;

                                                window.paint_quad(PaintQuad {
                                                    bounds: Bounds::centered_at(
                                                        center_pos,
                                                        mapped_size.into(),
                                                    ),
                                                    corner_radii: (x_radius.min(y_radius)).into(),
                                                    background: transparent_black().into(),
                                                    border_widths: Edges::all(px(1.0)),
                                                    border_color: white(),
                                                    border_style: gpui::BorderStyle::Solid,
                                                });
                                            }

                                            PanTiltSingleOriginEffectPresetType::Rect => {
                                                window.paint_quad(PaintQuad {
                                                    bounds: Bounds::centered_at(
                                                        center_pos,
                                                        mapped_size.into(),
                                                    ),
                                                    corner_radii: 0.0.into(),
                                                    background: transparent_black().into(),
                                                    border_widths: Edges::all(px(1.0)),
                                                    border_color: white(),
                                                    border_style: gpui::BorderStyle::Solid,
                                                });
                                            }
                                            PanTiltSingleOriginEffectPresetType::Figure8 => {
                                                // TODO:
                                            }
                                        }
                                    },
                                )),
                        )
                        .child(
                            v_flex()
                                .w_full()
                                .child(Select::new(&state.move_type_state))
                                .gap_4()
                                .child(
                                    v_flex()
                                        .w_full()
                                        .child(
                                            h_flex()
                                                .w_full()
                                                .text_sm()
                                                .gap_4()
                                                .child("X-Size")
                                                .child(Slider::new(&state.width_state)),
                                        )
                                        .child(
                                            h_flex()
                                                .w_full()
                                                .text_sm()
                                                .gap_4()
                                                .child("X-Size")
                                                .child(Slider::new(&state.height_state)),
                                        ),
                                ),
                        ),
                )
            })
            .child(
                Button::new("submit")
                    .label("Apply")
                    .on_click(cx.listener(Self::apply_preset)),
            )
    }
}

impl EditWindowDelegate for PresetPanTiltSingleOriginWindow {
    fn window_title(
        &self,
        _window: &mut gpui::Window,
        _cx: &gpui::App,
    ) -> impl Into<gpui::SharedString> {
        "Pan/Tilt Single Origin"
    }

    fn window_bounds(cx: &mut gpui::App) -> Option<gpui::WindowBounds> {
        Some(WindowBounds::centered(size(800.0.into(), 400.0.into()), cx))
    }

    fn handle_save(&self, _window: &mut gpui::Window, _cx: &mut gpui::App) {}

    fn handle_discard(&self, _window: &mut gpui::Window, _cx: &mut gpui::App) {}
}
