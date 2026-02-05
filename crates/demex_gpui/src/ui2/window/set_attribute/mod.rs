use std::collections::HashMap;

use demex_core::{
    channel3::attribute::FixtureChannel3Attribute,
    command::parser::nodes::{
        action::{Action, functions::set_function::SetAttributeChannelSetArgs},
        fixture_selector::FixtureSelector,
    },
    fixture::FixturePath,
};
use gpui::{
    App, AppContext, Context, Entity, InteractiveElement, ParentElement, Render, SharedString,
    Styled, Subscription, WindowBounds, prelude::FluentBuilder, size,
};
use gpui_component::{
    button::{Button, ButtonVariant, ButtonVariants},
    h_flex,
    scroll::ScrollableElement,
    v_flex,
};

use crate::{
    engine::{DemexEngineHandler, state::DemexUiState},
    ui2::{ext::GpuiContextExtension, wm::edit_window::EditWindowDelegate},
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

pub struct SetAttributeWindow {
    attribute: FixtureChannel3Attribute,

    channel_sets: Entity<HashMap<SharedString, Vec<FixturePath>>>,

    _subscriptions: Vec<Subscription>,
}

impl SetAttributeWindow {
    pub fn new(attribute: FixtureChannel3Attribute, cx: &mut Context<Self>) -> Self {
        let channel_sets = cx.new(|_| HashMap::new());

        let _subscriptions = vec![
            cx.observe(&DemexUiState::fixture_selection(cx), |this, _, cx| {
                this.update_channel_sets(cx);
            }),
            cx.observe_and_notify(&channel_sets),
        ];

        let s = Self {
            attribute,
            channel_sets,
            _subscriptions,
        };

        s.update_channel_sets(cx);

        s
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

impl Render for SetAttributeWindow {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        h_flex()
            .size_full()
            .p_4()
            .gap_4()
            .child(v_flex().size_full().flex_grow())
            .child(
                v_flex()
                    .id("set-attribute-channel-sets")
                    .overflow_y_scrollbar()
                    .size_full()
                    .gap_2()
                    .flex_grow()
                    .children(self.channel_sets.read(cx).iter().map(|(set, fixtures)| {
                        let all_fixtures_have_set = DemexUiState::fixture_selection(cx)
                            .read(cx)
                            .as_ref()
                            .is_some_and(|sel| sel.selection().fixtures().len() == fixtures.len());

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
                    })),
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
