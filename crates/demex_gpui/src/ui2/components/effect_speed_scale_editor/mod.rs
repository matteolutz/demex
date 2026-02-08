use demex_core::effect::speed::EffectSpeedScale;
use gpui::{App, AppContext, Context, Entity, Render, Styled, Subscription, Window};
use gpui_component::{
    IndexPath,
    select::{Select, SelectEvent, SelectItem, SelectState},
};
use itertools::Itertools;
use strum::IntoEnumIterator;

#[derive(Clone)]
struct EffectSpeedScaleSelectItem(EffectSpeedScale);

impl From<EffectSpeedScale> for EffectSpeedScaleSelectItem {
    fn from(value: EffectSpeedScale) -> Self {
        Self(value)
    }
}

impl SelectItem for EffectSpeedScaleSelectItem {
    type Value = EffectSpeedScale;

    fn title(&self) -> gpui::SharedString {
        match self.0 {
            EffectSpeedScale::Div128 => "/128",
            EffectSpeedScale::Div64 => "/64",
            EffectSpeedScale::Div32 => "/32",
            EffectSpeedScale::Div16 => "/16",
            EffectSpeedScale::Div8 => "/8",
            EffectSpeedScale::Div4 => "/4",
            EffectSpeedScale::Div2 => "/2",

            EffectSpeedScale::One => "One",

            EffectSpeedScale::Mul2 => "*2",
            EffectSpeedScale::Mul4 => "*4",
            EffectSpeedScale::Mul8 => "*8",
            EffectSpeedScale::Mul16 => "*16",
            EffectSpeedScale::Mul32 => "*32",
            EffectSpeedScale::Mul64 => "*64",
            EffectSpeedScale::Mul128 => "*128",
        }
        .into()
    }

    fn value(&self) -> &Self::Value {
        &self.0
    }
}

pub struct EffectSpeedScaleEditor {
    select_state: Entity<SelectState<Vec<EffectSpeedScaleSelectItem>>>,

    _subscriptions: Vec<Subscription>,
}

impl EffectSpeedScaleEditor {
    pub fn new(value: EffectSpeedScale, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let select_state = cx.new(|cx| {
            SelectState::new(
                EffectSpeedScale::iter().map_into().collect(),
                EffectSpeedScale::iter()
                    .find_position(|s| *s == value)
                    .map(|(idx, _)| IndexPath::new(idx)),
                window,
                cx,
            )
        });

        let _subscriptions = vec![cx.subscribe(
            &select_state,
            |_, _, evt: &SelectEvent<Vec<EffectSpeedScaleSelectItem>>, cx| match evt {
                SelectEvent::Confirm(_) => {
                    cx.notify();
                }
            },
        )];

        Self {
            select_state,
            _subscriptions,
        }
    }

    pub fn value(&self, cx: &App) -> Option<EffectSpeedScale> {
        self.select_state.read(cx).selected_value().copied()
    }
}

impl Render for EffectSpeedScaleEditor {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        _cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        Select::new(&self.select_state).w_full()
    }
}
