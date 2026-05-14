use gpui::{
    App, Context, ParentElement, Render, Styled, Subscription, Window, div, prelude::FluentBuilder,
};
use gpui_component::{
    IconName,
    button::Button,
    h_flex,
    scroll::ScrollableElement,
    tab::{Tab, TabBar},
    v_flex,
};

use crate::{
    engine::state::DemexUiState,
    storage,
    ui2::{
        window::{add_fixture::AddFixtureWindow, patch::universe::DmxUniverseOverview},
        wm::{
            WindowManager,
            edit_window::{EditWindowDelegate, WindowManagerExtension},
        },
    },
};

mod universe;

pub(super) fn init(_cx: &mut App) {}

pub struct PatchWindow {
    dmx_universes: Vec<u16>,
    selected_dmx_universe_tab: usize,

    _subscriptions: Vec<Subscription>,
}

impl PatchWindow {
    pub fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        let dmx_universes = DemexUiState::patch(cx).read(cx).dmx_universes().collect();

        let _subscriptions = vec![cx.observe(&DemexUiState::patch(cx), |this, patch, cx| {
            this.dmx_universes = patch.read(cx).dmx_universes().collect();
            cx.notify();
        })];

        Self {
            dmx_universes,
            selected_dmx_universe_tab: 0,
            _subscriptions,
        }
    }
}

impl EditWindowDelegate for PatchWindow {
    fn window_title(&self, _window: &mut gpui::Window, _cx: &App) -> impl Into<gpui::SharedString> {
        "Patch"
    }

    fn should_reactivate() -> bool {
        true
    }

    fn handle_save(&self, _window: &mut gpui::Window, _cx: &mut App) {}

    fn handle_discard(&self, _window: &mut gpui::Window, _cx: &mut App) {}

    fn should_have_save_button(_cx: &App) -> bool
    where
        Self: Sized,
    {
        false
    }
}

impl Render for PatchWindow {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        v_flex()
            .gap_2()
            .size_full()
            .child(
                h_flex()
                    .p_4()
                    .gap_4()
                    .justify_between()
                    .child(
                        Button::new("open-fixture-type-dir")
                            .label("Open GDTF directory")
                            .icon(IconName::FolderOpen)
                            .on_click(|_, window, cx| {
                                let answer = window.prompt(
                                    gpui::PromptLevel::Info,
                                    "Restart after adding GDTF files",
                                    Some("In order for the new fixtures to be available, you need to restart demex."),
                                    &["Ok"],
                                    cx,
                                );

                                cx.spawn(async |cx| {
                                    let Ok(_) = answer.await else {
                                        return;
                                    };

                                    cx.update(|cx| cx.open_with_system(storage::fixture_types()));
                                }).detach();
                            }),
                    )
                    .child(
                        Button::new("add-fixture")
                            .label("Add Fixture")
                            .icon(IconName::Plus)
                            .on_click(|_, _, cx| {
                                WindowManager::open_edit_window::<AddFixtureWindow>(
                                    cx,
                                    |window, cx| AddFixtureWindow::new(window, cx),
                                );
                            }),
                    ),
            )
            .child(
                v_flex()
                    .size_full()
                    .child(
                        TabBar::new("dmx-universes")
                            .selected_index(self.selected_dmx_universe_tab)
                            .on_click(cx.listener(|this, &idx, _, cx| {
                                this.selected_dmx_universe_tab = idx;
                                cx.notify();
                            }))
                            .children(self.dmx_universes.iter().map(|universe| {
                                Tab::new().label(format!("Universe {}", universe))
                            })),
                    )
                    .when_some(
                        self.dmx_universes.get(self.selected_dmx_universe_tab),
                        |this, &universe| {
                            this.child(
                                div()
                                    .size_full()
                                    .p_2()
                                    .overflow_y_scrollbar()
                                    .child(DmxUniverseOverview::new(universe)),
                            )
                        },
                    ),
            )
    }
}
