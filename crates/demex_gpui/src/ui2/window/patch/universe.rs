use demex_core::fixture::FixturePath;
use demex_dmx::address::DmxAddress;
use gpui::{
    InteractiveElement, IntoElement, ParentElement, RenderOnce, StatefulInteractiveElement, Styled,
    div, prelude::FluentBuilder, px, rems,
};
use gpui_component::{ActiveTheme, StyledExt, h_flex, tooltip::Tooltip, v_flex};

use crate::engine::state::DemexUiState;

#[derive(IntoElement)]
pub struct DmxUniverseOverview {
    universe: u16,
}

impl DmxUniverseOverview {
    pub fn new(universe: u16) -> Self {
        Self { universe }
    }
}

impl RenderOnce for DmxUniverseOverview {
    fn render(self, _window: &mut gpui::Window, cx: &mut gpui::App) -> impl gpui::IntoElement {
        div()
            .grid()
            .grid_cols_max_content(32)
            .justify_start()
            .gap_1()
            .children((1..=512).map(|channel| {
                let address = DmxAddress {
                    universe: self.universe,
                    channel,
                };

                let function = DemexUiState::patch(cx)
                    .read(cx)
                    .dmx_map()
                    .get(&address)
                    .cloned();

                v_flex()
                    .id(format!("{}-{}", self.universe, channel))
                    .w_12()
                    .h_12()
                    .bg(if function.is_some() {
                        cx.theme().list_active
                    } else {
                        cx.theme().accent
                    })
                    .when(function.is_some(), |this| {
                        this.border_1().border_color(cx.theme().list_active_border)
                    })
                    .relative()
                    .child(
                        div()
                            .absolute()
                            .top_0()
                            .left_0()
                            .px(px(2.))
                            .child(channel.to_string())
                            .text_size(rems(0.5)),
                    )
                    .flex()
                    .justify_center()
                    .items_center()
                    .when_some(function, |this, (f_id, attribute)| {
                        this.child(div().text_sm().child(format!("{}", f_id)))
                            .child(
                                div()
                                    .w_full()
                                    .text_ellipsis()
                                    .text_xs()
                                    .child(format!("{}", attribute)),
                            )
                            .when_some(
                                DemexUiState::patch(cx)
                                    .read(cx)
                                    .fixture(&FixturePath::new(f_id))
                                    .ok(),
                                |this, fixture| {
                                    let fixture_name = fixture.name().to_string();

                                    this.tooltip(move |window, cx| {
                                        let fixture_name = fixture_name.clone();

                                        Tooltip::element(move |_, _| {
                                            let fixture_name = fixture_name.clone();

                                            v_flex()
                                                .gap_1()
                                                .child(
                                                    h_flex()
                                                        .gap_1()
                                                        .child(
                                                            div().font_bold().child(fixture_name),
                                                        )
                                                        .child(format!("({})", f_id)),
                                                )
                                                .child(format!("DMX {}", address))
                                                .child(attribute.to_string())
                                        })
                                        .build(window, cx)
                                    })
                                },
                            )
                    })
            }))
    }
}
