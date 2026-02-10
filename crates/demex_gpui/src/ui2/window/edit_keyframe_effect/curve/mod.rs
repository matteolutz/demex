use std::rc::Rc;

use demex_core::keyframe_effect::effect_keyframe_curve::KeyframeEffectKeyframeCurve;
use gpui::{
    App, AppContext, Context, Entity, ParentElement, Render, Styled, Subscription, Window,
    WindowBounds, size,
};
use gpui_component::{
    select::{Select, SelectEvent, SelectItem, SelectState},
    v_flex,
};
use itertools::Itertools;
use strum::IntoEnumIterator;

use crate::ui2::wm::edit_window::EditWindowDelegate;

#[derive(Debug, Copy, Clone)]
struct KeyframeEffectCurveSelectItem(KeyframeEffectKeyframeCurve);

impl From<KeyframeEffectKeyframeCurve> for KeyframeEffectCurveSelectItem {
    fn from(value: KeyframeEffectKeyframeCurve) -> Self {
        Self(value)
    }
}

impl SelectItem for KeyframeEffectCurveSelectItem {
    type Value = KeyframeEffectKeyframeCurve;

    fn title(&self) -> gpui::SharedString {
        format!("{:?}", self.0).into()
    }

    fn value(&self) -> &Self::Value {
        &self.0
    }
}

pub struct EditKeyframeEffectCurveWindow {
    select_state: Entity<SelectState<Vec<KeyframeEffectCurveSelectItem>>>,

    on_change: Rc<dyn Fn(&KeyframeEffectKeyframeCurve, &mut Window, &mut App)>,

    _subscriptions: Vec<Subscription>,
}

impl EditKeyframeEffectCurveWindow {
    pub fn new(
        on_change: impl Fn(&KeyframeEffectKeyframeCurve, &mut Window, &mut App) + 'static,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let select_state = cx.new(|cx| {
            SelectState::new(
                KeyframeEffectKeyframeCurve::iter().map_into().collect(),
                None,
                window,
                cx,
            )
        });

        let _subscriptions = vec![cx.subscribe_in(
            &select_state,
            window,
            |this, _, evt: &SelectEvent<Vec<KeyframeEffectCurveSelectItem>>, window, cx| match evt {
                SelectEvent::Confirm(value) => {
                    if let Some(value) = value {
                        (this.on_change)(value, window, cx);
                        this.discard_and_close(cx);
                    }
                }
            },
        )];

        Self {
            select_state,
            on_change: Rc::new(on_change),
            _subscriptions,
        }
    }
}

impl Render for EditKeyframeEffectCurveWindow {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        _cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        v_flex()
            .p_4()
            .gap_4()
            .size_full()
            .justify_center()
            .items_center()
            .child(Select::new(&self.select_state))
    }
}

impl EditWindowDelegate for EditKeyframeEffectCurveWindow {
    fn window_title(
        &self,
        _window: &mut gpui::Window,
        _cx: &gpui::App,
    ) -> impl Into<gpui::SharedString> {
        "Edit Keyframe Curve"
    }

    fn window_bounds(cx: &mut gpui::App) -> Option<gpui::WindowBounds> {
        Some(WindowBounds::centered(size(350.0.into(), 250.0.into()), cx))
    }

    fn handle_save(&self, _window: &mut gpui::Window, _cx: &mut gpui::App) {}

    fn handle_discard(&self, _window: &mut gpui::Window, _cx: &mut gpui::App) {}
}
