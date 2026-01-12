use std::cmp::Ordering;

use demex_core::channel3::feature::feature_group::FixtureChannel3FeatureGroup;
use gpui::{
    App, AppContext, ClickEvent, Context, Entity, EventEmitter, FocusHandle, Focusable,
    IntoElement, ParentElement, Render, SharedString, Styled, Subscription, Window, div,
    prelude::FluentBuilder,
};
use gpui_component::{
    ActiveTheme, Disableable, IconName, Sizable,
    button::{Button, ButtonVariant, ButtonVariants},
    dock::{Panel, PanelEvent, register_panel},
    slider::Slider,
    tab::TabBar,
};
use itertools::Itertools;

use crate::{
    engine::state::DemexUiState,
    ui2::{
        config::AppConfigExt,
        panels::{
            attribute_editor::{
                attribute_state::AttributeEditorAttributeState,
                value_display::AttributeValueDisplayMode,
            },
            toolbar_buttons,
        },
    },
};

mod attribute_state;
mod value_display;

const ATTRIBUTE_EDTIOR_PANEL_NAME: &str = "demex-attribute-editor";

pub(super) fn register(cx: &mut App) {
    register_panel(cx, ATTRIBUTE_EDTIOR_PANEL_NAME, |_, _, _, window, cx| {
        Box::new(cx.new(|cx| AttributeEditorPanel::new(window, cx)))
    });
}

const ATTRIBUTE_PAGE_SIZE: usize = 5;

pub struct AttributeEditorPanel {
    focus_handle: FocusHandle,

    attributes: Entity<
        Vec<(
            Option<FixtureChannel3FeatureGroup>,
            Vec<AttributeEditorAttributeState>,
        )>,
    >,

    selected_tab: Entity<usize>,
    selected_tab_page: Entity<usize>,

    value_display_mode: Entity<AttributeValueDisplayMode>,

    _subscriptions: Vec<Subscription>,
}

impl AttributeEditorPanel {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let selected_tab = cx.new(|_| 0);
        let selected_tab_page = cx.new(|_| 0);

        let _subscriptions = vec![
            cx.observe_in(&DemexUiState::patch(cx), window, |this, _, window, cx| {
                this.get_attributes(window, cx);
            }),
            cx.observe_in(
                &DemexUiState::fixture_selection(cx),
                window,
                |this, _, window, cx| {
                    this.get_attributes(window, cx);
                },
            ),
            cx.observe(&selected_tab, |this, _, cx| {
                this.selected_tab_page.update(cx, |page, _| *page = 0);
                cx.notify();
            }),
        ];

        Self {
            focus_handle: cx.focus_handle(),
            attributes: cx.new(|_| Vec::new()),
            selected_tab,
            selected_tab_page,
            value_display_mode: cx.new(|_| Default::default()),
            _subscriptions,
        }
    }

    pub fn get_attributes(&self, window: &mut Window, cx: &mut Context<Self>) {
        let patch = DemexUiState::patch(cx);
        let fixture_selection = DemexUiState::fixture_selection(cx);

        let Some(fs) = fixture_selection.read(cx) else {
            return;
        };

        let new_attributes = fs
            .selection()
            .get_attributes(patch.read(cx))
            .into_iter()
            .sorted_by(|a, b| match (a.feature_group(), b.feature_group()) {
                (Some(a), Some(b)) => a.cmp(&b),
                (Some(_), None) => Ordering::Less,
                (None, Some(_)) => Ordering::Greater,
                (None, None) => Ordering::Equal,
            })
            .chunk_by(|attr| attr.feature_group())
            .into_iter()
            .map(|(k, group)| {
                (
                    k,
                    group
                        .sorted()
                        .map(|attr| AttributeEditorAttributeState::new(attr, window, cx))
                        .collect::<Vec<_>>(),
                )
            })
            .collect();

        self.attributes.update(cx, |attributes, _| {
            *attributes = new_attributes;
        });

        self.selected_tab.update(cx, |tab, cx| {
            *tab = 0;
            cx.notify();
        });

        cx.notify();
    }
}

impl EventEmitter<PanelEvent> for AttributeEditorPanel {}
impl Focusable for AttributeEditorPanel {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Panel for AttributeEditorPanel {
    fn panel_name(&self) -> &'static str {
        ATTRIBUTE_EDTIOR_PANEL_NAME
    }

    fn title(&mut self, _window: &mut gpui::Window, _cx: &mut Context<Self>) -> impl IntoElement {
        "Attribute Editor"
    }

    fn inner_padding(&self, _cx: &App) -> bool {
        false
    }

    fn toolbar_buttons(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<Vec<Button>> {
        Some(toolbar_buttons(self, window, cx))
    }
}

impl AttributeEditorPanel {
    fn get_num_pages(&self, cx: &App) -> usize {
        let Some((_, attributes)) = self.attributes.read(cx).get(*self.selected_tab.read(cx))
        else {
            return 0;
        };

        (attributes.len() + ATTRIBUTE_PAGE_SIZE - 1) / ATTRIBUTE_PAGE_SIZE
    }

    fn has_next_page(&self, cx: &App) -> bool {
        let Some((_, attributes)) = self.attributes.read(cx).get(*self.selected_tab.read(cx))
        else {
            return false;
        };

        let page = *self.selected_tab_page.read(cx);
        page < attributes.len() / ATTRIBUTE_PAGE_SIZE
    }

    fn handle_prev_page(&mut self, _: &ClickEvent, _: &mut Window, cx: &mut Context<Self>) {
        self.selected_tab_page.update(cx, |page, _| {
            if *page > 0 {
                *page -= 1
            }
        });
        cx.notify();
    }

    fn handle_next_page(&mut self, _: &ClickEvent, _: &mut Window, cx: &mut Context<Self>) {
        let has_next_page = self.has_next_page(cx);
        self.selected_tab_page.update(cx, |page, _| {
            if has_next_page {
                *page += 1;
            }
        });
        cx.notify();
    }

    fn render_selected_attributes(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let Some((_, attributes)) = self.attributes.read(cx).get(*self.selected_tab.read(cx))
        else {
            return div().child("-");
        };

        let page = *self.selected_tab_page.read(cx);

        div()
            .flex_1()
            .h_full()
            .grid()
            .grid_rows(1)
            .grid_cols(ATTRIBUTE_PAGE_SIZE as u16)
            .gap_1()
            .children(
                attributes
                    .iter()
                    .skip(page * ATTRIBUTE_PAGE_SIZE)
                    .take(ATTRIBUTE_PAGE_SIZE)
                    .map(|attr| {
                        div()
                            .bg(cx.theme().accent)
                            .rounded(cx.theme().radius)
                            .size_full()
                            .p_2()
                            .flex()
                            .flex_col()
                            .justify_between()
                            .child(attr.attribute.to_string())
                            .child(
                                div()
                                    .when(
                                        attr.value
                                            .read(cx)
                                            .as_ref()
                                            .is_none_or(|val| !val.is_home()),
                                        |div| div.text_color(cx.theme().yellow),
                                    )
                                    .child(if attr.value.read(cx).is_some() {
                                        self.value_display_mode.read(cx).format_value(
                                            attr.slider_state.read(cx).value().start(),
                                        )
                                    } else {
                                        "-".to_string()
                                    }),
                            )
                            .child(
                                Button::new(SharedString::from(format!("home-{}", attr.attribute)))
                                    .with_variant(ButtonVariant::Primary)
                                    .label("Home")
                                    .on_click({
                                        let attr = attr.attribute;
                                        move |_, _, cx| {
                                            AttributeEditorAttributeState::set_value(attr, None, cx)
                                        }
                                    }),
                            )
                            .child(div().flex_1())
                            .child(
                                Slider::new(&attr.slider_state)
                                    .with_size(cx.ui_config().ui_size())
                                    .flex_none()
                                    .w_full()
                                    .px_2(),
                            )
                    }),
            )
    }
}

impl Render for AttributeEditorPanel {
    fn render(
        &mut self,
        window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .child(
                TabBar::new("tabs")
                    .with_size(cx.ui_config().ui_size())
                    .selected_index(*self.selected_tab.read(cx))
                    .on_click(cx.listener(|this, tab, _, cx| {
                        this.selected_tab.update(cx, |selected_tab, cx| {
                            *selected_tab = *tab;
                            cx.notify();
                        });
                        cx.notify();
                    }))
                    .children(self.attributes.read(cx).iter().map(|(group, _)| {
                        group
                            .map(|k| k.name().to_string())
                            .unwrap_or_else(|| "Other".to_string())
                    })),
            )
            .child(
                div()
                    .p_1()
                    .gap_1()
                    .flex_1()
                    .flex()
                    .flex_row()
                    .items_center()
                    .child(
                        div()
                            .gap_1()
                            .flex()
                            .flex_col()
                            .items_center()
                            .w_16()
                            .child(format!(
                                "{}/{}",
                                self.selected_tab_page.read(cx) + 1,
                                self.get_num_pages(cx)
                            ))
                            .child(
                                Button::new("prev-page")
                                    .w_full()
                                    .icon(IconName::ChevronUp)
                                    .disabled(*self.selected_tab_page.read(cx) == 0)
                                    .on_click(cx.listener(Self::handle_prev_page))
                                    .w_full(),
                            )
                            .child(
                                Button::new("next-page")
                                    .icon(IconName::ChevronDown)
                                    .disabled(!self.has_next_page(cx))
                                    .on_click(cx.listener(Self::handle_next_page))
                                    .w_full(),
                            )
                            .child(
                                Button::new("change-value-display-mode")
                                    .label(self.value_display_mode.read(cx).indicator())
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.value_display_mode
                                            .update(cx, |mode, _| *mode = mode.next());
                                        cx.notify();
                                    }))
                                    .w_full(),
                            ),
                    )
                    .child(self.render_selected_attributes(window, cx)),
            )
    }
}
