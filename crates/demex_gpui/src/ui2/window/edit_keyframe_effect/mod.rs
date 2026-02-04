use std::{collections::HashMap, rc::Rc};

use demex_core::{
    channel3::attribute::FixtureChannel3Attribute,
    command::parser::nodes::action::Action,
    engine::comm::KeyframeEffectRequest,
    keyframe_effect::{effect::KeyframeEffect, effect_keyframe_curve::KeyframeEffectKeyframeCurve},
    presets::preset::FixturePresetId,
};
use gpui::{
    App, AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Subscription,
    Window, WindowBounds, div, point, size,
};
use gpui_component::v_flex;

use crate::{
    engine::DemexEngineHandler,
    ui2::{
        components::wave_editor::{
            Wave, WaveEasingFunction, WaveEasingMode, WaveEditor, WaveEditorEvent, WaveEditorState,
            WaveSegment,
        },
        wm::edit_window::EditWindowDelegate,
    },
};

mod actions {
    use gpui::{App, KeyBinding};

    pub const CONTEXT: &str = "demex-edit-keyframe-effect-window";

    gpui::actions!([QuitEditKeyframeEffect]);

    pub fn init(cx: &mut App) {
        cx.bind_keys([KeyBinding::new(
            "escape",
            QuitEditKeyframeEffect,
            Some(CONTEXT),
        )]);
    }
}

pub(super) fn init(cx: &mut App) {
    actions::init(cx);
}

impl WaveEasingFunction for KeyframeEffectKeyframeCurve {
    fn get_easing_mode(
        &self,
        from: gpui::Point<gpui::Pixels>,
        to: gpui::Point<gpui::Pixels>,
    ) -> WaveEasingMode {
        match self {
            Self::EaseIn => {
                let x_diff = to.x - from.x;

                let a = point(from.x + (0.11 * x_diff), from.y);
                let b = point(from.x + (0.5 * x_diff), from.y);
                (a, b).into()
            }
            Self::EaseOut => {
                let x_diff = to.x - from.x;

                let a = point(from.x + (0.5 * x_diff), to.y);
                let b = point(from.x + (0.89 * x_diff), to.y);
                (a, b).into()
            }
            Self::EaseInOut => {
                let x_diff = to.x - from.x;

                let a = point(from.x + (0.45 * x_diff), from.y);
                let b = point(from.x + (0.55 * x_diff), to.y);
                (a, b).into()
            }
            Self::Snap => WaveEasingMode::Snap,
            Self::Linear => WaveEasingMode::Linear,
        }
    }
}

pub struct EditKeyframeEffectWindow {
    preset_id: FixturePresetId,

    effect: Entity<Option<KeyframeEffect>>,
    waves: Vec<HashMap<FixtureChannel3Attribute, Entity<WaveEditorState>>>,

    _subscriptions: Vec<Subscription>,
}

impl EditKeyframeEffectWindow {
    pub fn new(preset_id: impl Into<FixturePresetId>, cx: &mut Context<Self>) -> Self {
        let preset_id = preset_id.into();

        let effect: Entity<Option<KeyframeEffect>> = cx.new(|_| None);

        DemexEngineHandler::send_with(
            cx.entity(),
            cx,
            KeyframeEffectRequest { preset_id },
            |effect_res, this, cx| {
                let Some(effect_res) = effect_res else {
                    return;
                };

                this.effect.update(cx, |effect, _| {
                    *effect = Some(effect_res.clone());
                });

                let waves = effect_res
                    .layers()
                    .iter()
                    .enumerate()
                    .map(|(layer_idx, layer)| {
                        let all_attributes = layer.attributes();
                        all_attributes
                            .into_iter()
                            .map(|attribute| {
                                let wave_segments = layer
                                    .keyframes()
                                    .iter()
                                    .map(|keyframe| {
                                        let attribute_values = keyframe
                                            .values_for_attribute(&attribute)
                                            .into_iter()
                                            .map(|val| val.as_f32())
                                            .collect();

                                        WaveSegment {
                                            starting_point: keyframe.starting_point(),
                                            values: attribute_values,
                                            easing_functions: Rc::new(keyframe.curve()),
                                        }
                                    })
                                    .collect::<Vec<_>>();

                                let wave = Wave {
                                    segments: wave_segments,
                                };

                                let wave_state = cx.new(|_| WaveEditorState::new(wave));

                                this._subscriptions.push(cx.subscribe(
                                    &wave_state,
                                    move |this, _, evt, cx| {
                                        this.set_edited(true, cx);

                                        match evt {
                                            WaveEditorEvent::WaveSegmentStartingPointChanged {
                                                segment_idx,
                                                starting_point,
                                            } => {
                                                this.effect.update(cx, |effect, _| {
                                                    let layer =
                                                        &mut effect.as_mut().unwrap().layers_mut()
                                                            [layer_idx];
                                                    layer.keyframes_mut()[*segment_idx]
                                                        .set_starting_point(*starting_point);
                                                });
                                            }
                                            _ => {}
                                        }

                                        // TODO: setup subscription
                                        cx.notify();
                                    },
                                ));

                                (attribute, wave_state)
                            })
                            .collect::<HashMap<_, _>>()
                    })
                    .collect::<Vec<_>>();

                this.waves = waves;
                cx.notify();
            },
        );

        let _subscriptions = vec![];

        Self {
            preset_id,
            effect,
            waves: Vec::new(),
            _subscriptions,
        }
    }
}

impl EditKeyframeEffectWindow {
    fn render_layer(
        &self,
        wave: &HashMap<FixtureChannel3Attribute, Entity<WaveEditorState>>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> impl IntoElement {
        v_flex()
            .w_full()
            .gap_2()
            .children(wave.iter().map(|(attribute, wave_state)| {
                v_flex()
                    .w_full()
                    .gap_1()
                    .child(div().text_lg().child(attribute.to_string()))
                    .child(WaveEditor::new(wave_state))
            }))
    }
}

impl Render for EditKeyframeEffectWindow {
    fn render(
        &mut self,
        window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        v_flex()
            .size_full()
            .p_4()
            .gap_4()
            .children(self.waves.iter().enumerate().map(|(idx, wave)| {
                v_flex()
                    .gap_2()
                    .w_full()
                    .child(div().text_xl().child(format!("Layer {}", idx + 1)))
                    .child(self.render_layer(wave, window, cx))
            }))
    }
}

impl EditWindowDelegate for EditKeyframeEffectWindow {
    fn window_title(&self, _window: &mut gpui::Window, _cx: &App) -> impl Into<gpui::SharedString> {
        "Edit Keyframe Effect"
    }

    fn handle_save(&self, _window: &mut gpui::Window, cx: &mut App) {
        if let Some(effect) = self.effect.read(cx).as_ref() {
            DemexEngineHandler::engine(cx).exec_ui(Action::PresetUpdateKeyframeEffect(
                self.preset_id,
                effect.clone(),
            ));
        }
    }

    fn handle_discard(&self, _window: &mut gpui::Window, _cx: &mut App) {}

    fn window_bounds(cx: &mut App) -> Option<gpui::WindowBounds> {
        Some(WindowBounds::centered(
            size(1000.0.into(), 600.0.into()),
            cx,
        ))
    }
}
