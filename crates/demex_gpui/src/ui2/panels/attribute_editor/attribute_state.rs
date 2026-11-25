use demex_core::{
    channel3::{
        attribute::FixtureChannel3Attribute, channel_value_discrete::FixtureChannelDiscreteValue,
        clamped_value::ClampedValue,
    },
    command::parser::nodes::{
        action::{Action, functions::set_function::SetAttributeValueArgs},
        fixture_selector::FixtureSelector,
    },
};
use gpui::{App, AppContext, Context, Entity, Subscription, Window};
use gpui_component::slider::{SliderEvent, SliderState};

use crate::{
    engine::{DemexEngineHandler, state::DemexUiState},
    ui2::panels::attribute_editor::AttributeEditorPanel,
};

#[derive(Debug)]
pub struct AttributeEditorAttributeState {
    pub(crate) attribute: FixtureChannel3Attribute,
    pub(crate) slider_state: Entity<SliderState>,
    pub(crate) value: Entity<Option<FixtureChannelDiscreteValue>>,
    _subscriptions: Vec<Subscription>,
}

impl AttributeEditorAttributeState {
    fn get_discrete_value(
        attribute: &FixtureChannel3Attribute,
        cx: &App,
    ) -> Option<FixtureChannelDiscreteValue> {
        let master_fixture = DemexUiState::fixture_selection(cx)
            .read(cx)
            .as_ref()
            .and_then(|sel| sel.selection().master_fixture().copied());

        master_fixture.as_ref().and_then(|fixture| {
            DemexUiState::fixture_values(cx)
                .read(cx)
                .get(fixture)
                .and_then(|values| values.get(attribute))
                .and_then(|value| value.try_as_discrete().cloned())
        })
    }

    fn get_clamped_value(
        value: Option<&FixtureChannelDiscreteValue>,
        attribute: &FixtureChannel3Attribute,
        cx: &App,
    ) -> Option<ClampedValue> {
        let master_fixture = DemexUiState::fixture_selection(cx)
            .read(cx)
            .as_ref()
            .and_then(|sel| sel.selection().master_fixture().copied());

        let cf = master_fixture.as_ref().and_then(|fixture| {
            DemexUiState::patch(cx)
                .read(cx)
                .fixture(fixture)
                .ok()
                .and_then(|fixture| fixture.channel_function(attribute))
        });

        value.zip(cf).map(|(value, cf)| value.to_clamped(cf))
    }
}

impl AttributeEditorAttributeState {
    pub fn set_value(attribute: FixtureChannel3Attribute, value: Option<f32>, cx: &App) {
        DemexEngineHandler::engine(cx).exec_ui(Action::SetAttributeValue(SetAttributeValueArgs {
            fixture_selector: FixtureSelector::current_fixtures_selected(),
            attribute,
            attribute_value: value.map(|v| v.into()),
        }));
    }

    pub fn new(
        attribute: FixtureChannel3Attribute,
        window: &mut Window,
        cx: &mut Context<AttributeEditorPanel>,
    ) -> Self {
        let value = Self::get_discrete_value(&attribute, cx);
        let clamped_value = Self::get_clamped_value(value.as_ref(), &attribute, cx);

        let value_entity = cx.new(|_| value);
        let slider_state = cx.new(|_| {
            SliderState::new()
                .min(0.0)
                .max(1.0)
                .step(0.001)
                .default_value(clamped_value.map(|cv| cv.as_f32()).unwrap_or(0.0))
        });

        let _subscriptions = vec![
            cx.observe_in(&DemexUiState::fixture_values(cx), window, {
                let slider_state = slider_state.clone();
                let value_entity = value_entity.clone();

                move |_, _, window, cx| {
                    let value = Self::get_discrete_value(&attribute, cx);
                    let clamped_value = Self::get_clamped_value(value.as_ref(), &attribute, cx);

                    slider_state.update(cx, |slider_state, cx| {
                        slider_state.set_value(
                            clamped_value.map(|cv| cv.as_f32()).unwrap_or(0.0),
                            window,
                            cx,
                        );
                        // Don't notify the SliderState context right here
                        // this would cause an infinite loop
                    });
                    value_entity.update(cx, |value_entity, cx| {
                        *value_entity = value;
                        cx.notify();
                    });

                    cx.notify();
                }
            }),
            cx.subscribe(&slider_state, move |_, _, evt, cx| match evt {
                SliderEvent::Change(value) => {
                    Self::set_value(attribute, Some(value.start().into()), cx);
                }
            }),
        ];

        Self {
            attribute,
            slider_state,
            value: value_entity,
            _subscriptions,
        }
    }
}
