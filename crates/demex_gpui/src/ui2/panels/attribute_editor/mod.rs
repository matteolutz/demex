use std::{cmp::Ordering, collections::HashMap};

use demex_core::{
    channel3::feature::feature_group::FixtureChannel3FeatureGroup,
    command::parser::nodes::action::Action,
};
use gpui::{
    App, AppContext, ClickEvent, Context, Entity, EventEmitter, FocusHandle, Focusable,
    IntoElement, ParentElement, Render, SharedString, Styled, Subscription, Window, div,
    prelude::FluentBuilder,
};
use gpui_component::{
    ActiveTheme, Disableable, IconName, Sizable,
    button::{Button, ButtonVariant, ButtonVariants},
    dock::PanelEvent,
    h_flex,
    slider::Slider,
    tab::{Tab, TabBar},
};
use itertools::Itertools;

use crate::{
    engine::{DemexEngineHandler, state::DemexUiState},
    ui2::{
        config::AppConfigExt,
        icon::DemexIconName,
        panels::{
            DemexPanel,
            attribute_editor::{
                attribute_state::AttributeEditorAttributeState,
                value_display::AttributeValueDisplayMode,
            },
        },
        window::set_attribute::SetAttributeWindow,
        wm::{WindowManager, edit_window::WindowManagerExtension},
    },
};

mod attribute_state;
mod value_display;

const ATTRIBUTE_PAGE_SIZE: usize = 5;

pub struct AttributeEditorPanel {
    focus_handle: FocusHandle,

    attributes:
        Entity<HashMap<Option<FixtureChannel3FeatureGroup>, Vec<AttributeEditorAttributeState>>>,

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
                this.visible_attributes_changed(cx);

                cx.notify();
            }),
            cx.observe(&selected_tab_page, |this, _, cx| {
                this.visible_attributes_changed(cx);
            }),
        ];

        Self {
            focus_handle: cx.focus_handle(),
            attributes: cx.new(|_| HashMap::new()),
            selected_tab,
            selected_tab_page,
            value_display_mode: cx.new(|_| Default::default()),
            _subscriptions,
        }
    }

    pub fn visible_attributes_changed(&self, cx: &mut Context<Self>) {
        let Some(attributes) = self
            .attributes
            .read(cx)
            .get(&self.get_selected_feature_group(cx))
        else {
            return;
        };

        let page = *self.selected_tab_page.read(cx);

        let visible_attributes = attributes
            .iter()
            .skip(page * ATTRIBUTE_PAGE_SIZE)
            .take(ATTRIBUTE_PAGE_SIZE)
            .map(|attribute| attribute.attribute)
            .collect();

        DemexEngineHandler::engine(cx)
            .exec_ui(Action::VisibleEncoderAttributesChanged(visible_attributes));
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

        if self.get_num_pages(cx) <= *self.selected_tab_page.read(cx) {
            self.selected_tab_page.update(cx, |page, _| {
                *page = 0;
            });
            cx.notify();
        }

        self.visible_attributes_changed(cx);

        cx.notify();
    }
}

impl EventEmitter<PanelEvent> for AttributeEditorPanel {}
impl Focusable for AttributeEditorPanel {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl DemexPanel for AttributeEditorPanel {
    fn panel_type() -> super::DockWindowPanelType {
        super::DockWindowPanelType::AttributeEditor
    }

    fn deserialize(
        _dock_area: gpui::WeakEntity<gpui_component::dock::DockArea>,
        _panel_state: &gpui_component::dock::PanelState,
        _panel_info: &gpui_component::dock::PanelInfo,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        AttributeEditorPanel::new(window, cx)
    }
}

impl AttributeEditorPanel {
    fn get_selected_feature_group(&self, cx: &App) -> Option<FixtureChannel3FeatureGroup> {
        FixtureChannel3FeatureGroup::iter_without_all()
            .skip(*self.selected_tab.read(cx))
            .next()
    }

    fn get_num_pages(&self, cx: &App) -> usize {
        let feature_group = self.get_selected_feature_group(cx);
        self.get_num_pages_for_feature_group(&feature_group, cx)
    }

    fn get_num_pages_for_feature_group(
        &self,
        feature_group: &Option<FixtureChannel3FeatureGroup>,
        cx: &App,
    ) -> usize {
        let Some(attributes) = self.attributes.read(cx).get(feature_group) else {
            return 0;
        };

        (attributes.len() + ATTRIBUTE_PAGE_SIZE - 1) / ATTRIBUTE_PAGE_SIZE
    }

    fn has_next_page(&self, cx: &App) -> bool {
        let Some(attributes) = self
            .attributes
            .read(cx)
            .get(&self.get_selected_feature_group(cx))
        else {
            return false;
        };

        let page = *self.selected_tab_page.read(cx);
        page < attributes.len() / ATTRIBUTE_PAGE_SIZE
    }

    fn handle_prev_page(&mut self, _: &ClickEvent, _: &mut Window, cx: &mut Context<Self>) {
        self.selected_tab_page.update(cx, |page, cx| {
            if *page > 0 {
                *page -= 1;
                cx.notify();
            }
        });
        cx.notify();
    }

    fn handle_next_page(&mut self, _: &ClickEvent, _: &mut Window, cx: &mut Context<Self>) {
        let has_next_page = self.has_next_page(cx);
        self.selected_tab_page.update(cx, |page, cx| {
            if has_next_page {
                *page += 1;
                cx.notify();
            }
        });
        cx.notify();
    }

    fn render_selected_attributes(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let Some(attributes) = self
            .attributes
            .read(cx)
            .get(&self.get_selected_feature_group(cx))
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
                            .child(
                                h_flex().w_full()
                                    .child(div().flex_grow().w_full().child(attr.attribute.to_string()))
                                    .child(
                                        Button::new(SharedString::from(format!(
                                            "home-{}",
                                            attr.attribute
                                        )))
                                        .icon(DemexIconName::Home)
                                        .with_variant(ButtonVariant::Primary)
                                        .on_click({
                                            let attr = attr.attribute;
                                            move |_, _, cx| {
                                                AttributeEditorAttributeState::set_value(
                                                    attr, None, cx,
                                                )
                                            }
                                        }),
                                    )
                                .child(Button::new(SharedString::from(format!(
                                    "edit-{}",
                                    attr.attribute
                                )))
                                .icon(DemexIconName::CallMade)
                                .with_variant(ButtonVariant::Secondary)
                                .on_click({
                                    let attr = attr.attribute;
                                    move |_, _, cx| {
                                        WindowManager::open_edit_window::<SetAttributeWindow>(cx, move |window, cx| SetAttributeWindow::new(attr, window, cx));
                                    }
                                })
                                )
                            )
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
                    .children(
                        FixtureChannel3FeatureGroup::iter_without_all().map(|group| {
                            Tab::new()
                                .flex_grow()
                                .disabled(
                                    self.get_num_pages_for_feature_group(&Some(group), cx) == 0,
                                )
                                .label(group.name().to_string())
                        }),
                    )
                    .child(
                        Tab::new()
                            .flex_grow()
                            .disabled(self.get_num_pages_for_feature_group(&None, cx) == 0)
                            .label("Other"),
                    ),
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
