use std::rc::Rc;

use demex_core::{
    channel3::attribute::FixtureChannel3Attribute,
    keyframe_effect::effect_layer::KeyframeEffectLayer,
};
use gpui::{
    AppContext, Context, Entity, EventEmitter, ParentElement, Render, Styled, Subscription, Window,
    div, prelude::FluentBuilder,
};
use gpui_component::{
    IconName, StyledExt,
    button::{Button, ButtonVariants},
    h_flex,
    input::{InputEvent, InputState, NumberInput},
    menu::DropdownMenu,
    tab::{Tab, TabBar},
    v_flex,
};
use itertools::Itertools;

use crate::ui2::components::wave_editor::{
    Wave, WaveEditor, WaveEditorEvent, WaveEditorState, WaveSegment,
};

pub enum EditKeyframeEffectLayerEvent {
    KeyframeStartingPointChanged {
        keyframe_idx: usize,
        starting_point: f32,
    },
    PhaseMultiplierChanged(f32),
}

pub struct EditKeyframeEffectLayer {
    selected_attribute: Option<usize>,

    wave: Vec<(FixtureChannel3Attribute, Entity<WaveEditorState>)>,
    phase_multiplier: Entity<InputState>,

    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<EditKeyframeEffectLayerEvent> for EditKeyframeEffectLayer {}
impl EditKeyframeEffectLayer {
    pub fn new(layer: &KeyframeEffectLayer, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let mut _subscriptions = Vec::new();

        let phase_multiplier = cx.new(|cx| {
            InputState::new(window, cx).default_value(layer.phase_multiplier().to_string())
        });

        _subscriptions.push(cx.subscribe(
            &phase_multiplier,
            |_, phase_multiplier, evt, cx| match evt {
                InputEvent::Change => {
                    if let Ok(phase_multiplier) = phase_multiplier.read(cx).value().parse() {
                        cx.emit(EditKeyframeEffectLayerEvent::PhaseMultiplierChanged(
                            phase_multiplier,
                        ))
                    }
                }
                _ => {}
            },
        ));

        let all_attributes = layer.attributes();
        let wave = all_attributes
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
                    phase_offset: 0.0,
                    phase_length: 2.0 * std::f32::consts::PI,
                };

                let wave_state = cx.new(|_| WaveEditorState::new(wave));

                _subscriptions.push(cx.subscribe(&wave_state, move |_, _, evt, cx| match evt {
                    &WaveEditorEvent::WaveSegmentStartingPointChanged {
                        segment_idx,
                        starting_point,
                    } => {
                        cx.emit(EditKeyframeEffectLayerEvent::KeyframeStartingPointChanged {
                            keyframe_idx: segment_idx,
                            starting_point,
                        });
                    }
                    _ => {}
                }));

                (attribute, wave_state)
            })
            .sorted_by_key(|(attribute, _)| *attribute)
            .collect::<Vec<_>>();

        let selected_attribute = (wave.len() > 0).then_some(0);

        Self {
            wave,
            selected_attribute,
            phase_multiplier,
            _subscriptions,
        }
    }
}

impl Render for EditKeyframeEffectLayer {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        v_flex()
            .w_full()
            .gap_2()
            .child(
                h_flex()
                    .w_full()
                    .gap_2()
                    .child(div().child("Phase multiplier").text_sm().font_bold())
                    .child(NumberInput::new(&self.phase_multiplier)),
            )
            .child(
                TabBar::new("selected-attribute")
                    .when_some(self.selected_attribute, |this, selected| {
                        this.selected_index(selected)
                    })
                    .on_click(cx.listener(|this, new_selected, _, cx| {
                        if *new_selected < this.wave.len() {
                            this.selected_attribute = Some(*new_selected);
                            cx.notify();
                        }
                    }))
                    .children(
                        self.wave
                            .iter()
                            .map(|(attr, _)| Tab::new().label(attr.to_string())),
                    )
                    .child(
                        Tab::new().child(
                            Button::new("add-attribute")
                                .ghost()
                                .icon(IconName::Plus)
                                .dropdown_menu(|menu, _, _| menu),
                        ),
                    ),
            )
            .when_some(self.selected_attribute, |this, selected| {
                this.child(WaveEditor::new(&self.wave[selected].1))
            })
    }
}
