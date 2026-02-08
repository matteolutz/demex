use std::rc::Rc;

use demex_core::{
    command::parser::nodes::action::Action,
    engine::comm::KeyframeEffectRequest,
    keyframe_effect::{
        effect_keyframe_curve::KeyframeEffectKeyframeCurve, effect_runtime::KeyframeEffectRuntime,
    },
    presets::preset::FixturePresetId,
    updatables::runtime::RuntimePhase,
};
use gpui::{
    App, AppContext, Context, Entity, InteractiveElement, ParentElement, Render, Styled,
    Subscription, Window, WindowBounds, div, point, size,
};
use gpui_component::{
    button::Button,
    menu::{DropdownMenu, PopupMenuItem},
    scroll::ScrollableElement,
    v_flex,
};
use strum::IntoEnumIterator;

use crate::{
    engine::DemexEngineHandler,
    ui2::{
        components::{
            runtime_phase_editor::RuntimePhaseEditor,
            wave_editor::{WaveEasingFunction, WaveEasingMode},
        },
        window::edit_keyframe_effect::layer::{
            EditKeyframeEffectLayer, EditKeyframeEffectLayerEvent,
        },
        wm::edit_window::EditWindowDelegate,
    },
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, strum::EnumIter)]
enum KeyframeEffectPresetTypes {
    PanTiltSingleOrigin,
}

impl std::fmt::Display for KeyframeEffectPresetTypes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PanTiltSingleOrigin => write!(f, "Pan/Tilt Single Origin"),
        }
    }
}

mod layer;

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

    effect: Entity<Option<KeyframeEffectRuntime>>,

    layers: Vec<Entity<EditKeyframeEffectLayer>>,

    runtime_phase_editor: Entity<RuntimePhaseEditor>,

    _subscriptions: Vec<Subscription>,
}

impl EditKeyframeEffectWindow {
    pub fn new(
        preset_id: impl Into<FixturePresetId>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let preset_id = preset_id.into();

        let effect: Entity<Option<KeyframeEffectRuntime>> = cx.new(|_| None);

        let runtime_phase_editor =
            cx.new(|cx| RuntimePhaseEditor::new(RuntimePhase::default(), window, cx));

        DemexEngineHandler::send_in_visual(
            window,
            cx,
            KeyframeEffectRequest { preset_id },
            |this, effect_res, window, cx| {
                let Some(effect_res) = effect_res else {
                    return;
                };

                this._subscriptions.clear();

                this.effect.update(cx, |effect, _| {
                    *effect = Some(effect_res.clone());
                });

                this.runtime_phase_editor.update(cx, |editor, cx| {
                    editor.set_runtime_phase(*effect_res.phase(), window, cx)
                });

                let layers = effect_res
                    .effect()
                    .layers()
                    .iter()
                    .enumerate()
                    .map(|(layer_idx, layer)| {
                        let editor = cx.new(|cx| EditKeyframeEffectLayer::new(layer, window, cx));

                        this._subscriptions.push(cx.subscribe(
                            &editor,
                            move |this, _, evt: &EditKeyframeEffectLayerEvent, cx| {
                                this.set_edited(true, cx);
                                match evt {
                                    &EditKeyframeEffectLayerEvent::PhaseMultiplierChanged(phase_multiplier) => {
                                        this.effect.update(cx, |effect, _| {
                                                let layer = &mut effect
                                                    .as_mut()
                                                    .unwrap()
                                                    .effect_mut()
                                                    .layers_mut()[layer_idx];
                                                *layer.phase_multiplier_mut() = phase_multiplier;
                                            });
                                    }
                                    &EditKeyframeEffectLayerEvent::KeyframeStartingPointChanged { keyframe_idx, starting_point } => {
                                        this.effect.update(cx, |effect, _| {
                                            let layer = &mut effect
                                                .as_mut()
                                                .unwrap()
                                                .effect_mut()
                                                .layers_mut()[layer_idx];
                                            layer.keyframes_mut()[keyframe_idx]
                                                .set_starting_point(starting_point);
                                        });
                                    }
                                }
                            },
                        ));

                        editor
                    })
                    .collect::<Vec<_>>();

                this.layers = layers;
                cx.notify();
            },
        );

        let _subscriptions =
            vec![
                cx.observe(&runtime_phase_editor, |this, runtime_phase_editor, cx| {
                    this.set_edited(true, cx);
                    this.effect.update(cx, |effect, cx| {
                        if let Some(effect) = effect {
                            *effect.phase_mut() = runtime_phase_editor.read(cx).runtime_phase();
                        }
                    });
                }),
            ];

        Self {
            preset_id,
            effect,
            layers: Vec::new(),
            runtime_phase_editor,
            _subscriptions,
        }
    }
}

impl Render for EditKeyframeEffectWindow {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        _cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        v_flex()
            .size_full()
            .p_4()
            .gap_8()
            .id("edit-keyframe-container")
            .overflow_y_scrollbar()
            .child(
                Button::new("use-preset")
                    .label("Use preset")
                    .dropdown_menu(|mut menu, _, _| {
                        for preset in KeyframeEffectPresetTypes::iter() {
                            menu = menu.item(PopupMenuItem::Item {
                                icon: None,
                                label: preset.to_string().into(),
                                disabled: false,
                                checked: false,
                                is_link: false,
                                action: None,
                                handler: Some(Rc::new(|_, _, _| {})),
                            });
                        }
                        menu
                    }),
            )
            .child(
                v_flex()
                    .w_full()
                    .gap_2()
                    .child(div().text_xl().child("Phase"))
                    .child(self.runtime_phase_editor.clone()),
            )
            .children(self.layers.iter().enumerate().map(|(idx, layer)| {
                v_flex()
                    .gap_2()
                    .w_full()
                    .child(div().text_xl().child(format!("Layer {}", idx + 1)))
                    .child(layer.clone())
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
            size(1200.0.into(), 800.0.into()),
            cx,
        ))
    }
}
