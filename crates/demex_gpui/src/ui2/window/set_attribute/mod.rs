use std::collections::HashMap;

use demex_core::{
    channel3::{attribute::FixtureChannel3Attribute, clamped_value::ClampedValue},
    command::parser::nodes::{
        action::{
            Action,
            functions::set_function::{SetAttributeChannelSetArgs, SetAttributeValueArgs},
        },
        fixture_selector::FixtureSelector,
    },
    fixture::FixturePath,
};
use gpui::{
    App, AppContext, Context, Entity, InteractiveElement, IntoElement, ParentElement, Render,
    SharedString, Styled, Subscription, Window, WindowBounds, div, prelude::FluentBuilder, size,
};
use gpui_component::{
    ActiveTheme,
    button::{Button, ButtonVariant, ButtonVariants},
    h_flex,
    input::{Input, InputEvent, InputState},
    scroll::ScrollableElement,
    v_flex,
};
use itertools::Itertools;

use crate::{
    engine::{DemexEngineHandler, state::DemexUiState},
    ui2::{
        components::number_input_grid::{NumberInputGrid, NumberInputGridEvent},
        ext::GpuiContextExtension,
        wm::edit_window::EditWindowDelegate,
    },
};

mod actions {
    use gpui::{App, KeyBinding};

    pub const CONTEXT: &str = "demex-set-attribute-window";

    gpui::actions!([QuitSetAttribute]);

    pub fn init(cx: &mut App) {
        cx.bind_keys([KeyBinding::new("escape", QuitSetAttribute, Some(CONTEXT))]);
    }
}

pub(super) fn init(cx: &mut App) {
    actions::init(cx);
}

#[derive(Debug, Copy, Clone)]
enum SetAttributeInputValue {
    Decimal(f32),
    Percent(f32),
    Byte(u8),
}

impl SetAttributeInputValue {
    pub fn parse(input_state: &Entity<InputState>, cx: &App) -> Option<Self> {
        let value = input_state.read(cx).value();

        // this means we have a byte value
        if let Some(byte_value_str) = value.strip_suffix("b") {
            return byte_value_str.parse::<u8>().ok().map(Self::Byte);
        }

        let value = value.parse::<f32>().ok()?.max(0.0);

        if value <= 1.0 {
            Some(Self::Decimal(value))
        } else {
            Some(Self::Percent(value.min(100.0)))
        }
    }

    pub fn to_clamped(self) -> ClampedValue {
        match self {
            Self::Decimal(val) => val.try_into().unwrap(),
            Self::Percent(val) => (val / 100.0).try_into().unwrap(),
            Self::Byte(val) => (val as f32 / 255.0).try_into().unwrap(),
        }
    }
}

pub struct SetAttributeWindow {
    attribute: FixtureChannel3Attribute,

    value_input_state: Entity<InputState>,
    channel_sets: Entity<HashMap<SharedString, Vec<FixturePath>>>,

    _subscriptions: Vec<Subscription>,
}

impl SetAttributeWindow {
    pub fn new(
        attribute: FixtureChannel3Attribute,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let channel_sets = cx.new(|_| HashMap::new());
        let value_input_state = cx.new(|cx| InputState::new(window, cx));

        let _subscriptions = vec![
            cx.subscribe(
                &value_input_state,
                |this, _, evt: &InputEvent, cx| match evt {
                    InputEvent::PressEnter { .. } => this.handle_input_state_submit(cx),
                    _ => {}
                },
            ),
            cx.observe(&DemexUiState::fixture_selection(cx), |this, _, cx| {
                this.update_channel_sets(cx);
            }),
            cx.observe_and_notify(&channel_sets),
        ];

        let s = Self {
            attribute,
            channel_sets,
            value_input_state,
            _subscriptions,
        };

        s.update_channel_sets(cx);

        s
    }

    fn handle_input_state_submit(&mut self, cx: &mut Context<Self>) {
        if let Some(value) = SetAttributeInputValue::parse(&self.value_input_state, cx) {
            let clamped_value = value.to_clamped();

            DemexEngineHandler::engine(cx).exec_ui(Action::SetAttributeValue(
                SetAttributeValueArgs {
                    fixture_selector: FixtureSelector::current_fixtures_selected(),
                    attribute: self.attribute,
                    attribute_value: Some(clamped_value.as_f32().into()),
                },
            ));
            self.close(cx);
        }
    }

    fn update_channel_sets(&self, cx: &mut Context<Self>) {
        self.channel_sets.update(cx, |sets, cx| {
            let Some(fixture_selection) = DemexUiState::fixture_selection(cx).read(cx) else {
                sets.clear();
                cx.notify();
                return;
            };

            let patch = DemexUiState::patch(cx).read(cx);

            sets.clear();

            for fixture in fixture_selection
                .selection()
                .fixtures()
                .iter()
                .filter_map(|f_path| patch.fixture(f_path).ok())
                .filter(|fixture| fixture.has_attribute(&self.attribute))
            {
                for set in fixture
                    .channel_function(&self.attribute)
                    .unwrap()
                    .channel_set_names()
                {
                    let fixtures_with_set = sets.entry(set.into()).or_default();
                    fixtures_with_set.push(fixture.path());
                }
            }

            cx.notify();
        });
    }

    fn set_channel_set(&self, set: SharedString, cx: &mut App) {
        DemexEngineHandler::engine(cx).exec_ui(Action::SetAttributeChannlSet(
            SetAttributeChannelSetArgs {
                fixture_selector: FixtureSelector::current_fixtures_selected(),
                attribute: self.attribute,
                channel_set: set.to_string(),
            },
        ));
        self.close(cx);
    }
}

impl SetAttributeWindow {
    fn render_number_input_grid(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        NumberInputGrid::new(3, 4)
            // buttons 1-9 and 0
            .buttons((1..=9).map(|number| SharedString::new(number.to_string())))
            .button(SharedString::new_static("."))
            .button(SharedString::new_static("0"))
            .button(SharedString::new_static("b"))
            .on_click(cx.listener(|this, evt, window, cx| match evt {
                NumberInputGridEvent::Insert(val) => {
                    this.value_input_state.update(cx, |state, cx| {
                        state.insert(val, window, cx);
                    });
                    cx.notify();
                }
                _ => {}
            }))
    }
}

impl Render for SetAttributeWindow {
    fn render(
        &mut self,
        window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        h_flex()
            .key_context(actions::CONTEXT)
            .on_action(cx.listener(|this, _: &actions::QuitSetAttribute, _, cx| {
                this.close(cx);
            }))
            .size_full()
            .p_4()
            .gap_6()
            .child(
                v_flex()
                    .size_full()
                    .flex_grow()
                    .gap_2()
                    .child(
                        h_flex()
                            .w_full()
                            .gap_1()
                            .child(Input::new(&self.value_input_state).w_full())
                            .child(h_flex().justify_center().w_6().child(
                                match SetAttributeInputValue::parse(&self.value_input_state, cx) {
                                    Some(value) => match value {
                                        SetAttributeInputValue::Byte(_) => div().child("8b"),
                                        SetAttributeInputValue::Decimal(_) => div().child(".2"),
                                        SetAttributeInputValue::Percent(_) => div().child("%"),
                                    },
                                    None => div().child("?").text_color(cx.theme().red),
                                },
                            )),
                    )
                    .child(self.render_number_input_grid(window, cx)),
            )
            .child(
                v_flex()
                    .id("set-attribute-channel-sets")
                    .overflow_y_scrollbar()
                    .size_full()
                    .gap_2()
                    .flex_grow()
                    .children(
                        self.channel_sets
                            .read(cx)
                            .iter()
                            .sorted_by_key(|(set, _)| set.as_str())
                            .map(|(set, fixtures)| {
                                let all_fixtures_have_set = DemexUiState::fixture_selection(cx)
                                    .read(cx)
                                    .as_ref()
                                    .is_some_and(|sel| {
                                        sel.selection().fixtures().len() == fixtures.len()
                                    });

                                let entity = cx.entity();

                                Button::new(format!("channel-set-{}", set.clone()))
                                    .when(all_fixtures_have_set, |this| {
                                        this.with_variant(ButtonVariant::Success)
                                    })
                                    .child(set.clone())
                                    .on_click({
                                        let set = set.clone();
                                        move |_, _, cx| {
                                            let set = set.clone();
                                            entity.update(cx, move |this, cx| {
                                                this.set_channel_set(set, cx);
                                            });
                                        }
                                    })
                                    .w_full()
                            }),
                    ),
            )
    }
}

impl EditWindowDelegate for SetAttributeWindow {
    fn window_title(&self, _window: &mut gpui::Window, _cx: &App) -> impl Into<gpui::SharedString> {
        format!("Set {}", self.attribute)
    }

    fn window_bounds(cx: &mut gpui::App) -> Option<gpui::WindowBounds> {
        Some(WindowBounds::centered(size(500.0.into(), 300.0.into()), cx))
    }

    fn handle_save(&self, _window: &mut gpui::Window, _cx: &mut App) {}

    fn handle_discard(&self, _window: &mut gpui::Window, _cx: &mut App) {}
}
