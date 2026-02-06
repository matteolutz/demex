use std::rc::Rc;

use gpui::{
    App, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div,
    prelude::FluentBuilder,
};
use gpui_component::{
    ActiveTheme, IconName,
    button::{Button, ButtonVariants},
};

#[derive(Debug, PartialEq, Eq)]
pub enum NumberInputGridEvent {
    Insert(SharedString),
    Delete,
    Submit,
}

#[derive(IntoElement)]
pub enum NumberInputGridButtonRender {
    Text(SharedString),
    Icon(IconName),
}

impl RenderOnce for NumberInputGridButtonRender {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        match self {
            Self::Text(text) => text.into_any_element(),
            Self::Icon(icon_name) => icon_name.into_any_element(),
        }
    }
}

impl NumberInputGridButtonRender {
    pub fn icon(icon_name: IconName) -> Self {
        Self::Icon(icon_name)
    }
}

impl From<SharedString> for NumberInputGridButton {
    fn from(value: SharedString) -> Self {
        Self::Insert(value)
    }
}

impl<T: Into<SharedString>> From<T> for NumberInputGridButtonRender {
    fn from(value: T) -> Self {
        Self::Text(value.into())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum NumberInputGridButton {
    Insert(SharedString),
    Delete,
    Submit,
    Empty,
}

impl NumberInputGridButton {
    pub fn insert(value: impl Into<SharedString>) -> Self {
        Self::Insert(value.into())
    }

    pub fn insert_char(value: char) -> Self {
        Self::insert(value.to_string())
    }

    fn render(self) -> Option<(NumberInputGridButtonRender, NumberInputGridEvent)> {
        match self {
            Self::Insert(value) => {
                Some((value.clone().into(), NumberInputGridEvent::Insert(value)))
            }
            Self::Delete => Some((
                NumberInputGridButtonRender::icon(IconName::Delete),
                NumberInputGridEvent::Delete,
            )),
            Self::Submit => Some(("Ok".into(), NumberInputGridEvent::Submit)),
            Self::Empty => None,
        }
    }
}

#[derive(IntoElement)]
pub struct NumberInputGrid {
    buttons: Vec<NumberInputGridButton>,
    cols: u16,
    rows: u16,

    on_click: Option<Rc<dyn Fn(&NumberInputGridEvent, &mut Window, &mut App)>>,
}

impl NumberInputGrid {
    pub fn new(cols: u16, rows: u16) -> Self {
        Self {
            buttons: vec![],
            on_click: None,
            cols,
            rows,
        }
    }

    pub fn button(mut self, button: impl Into<NumberInputGridButton>) -> Self {
        self.buttons.push(button.into());
        self
    }

    pub fn buttons<B, I>(mut self, buttons: I) -> Self
    where
        B: Into<NumberInputGridButton>,
        I: IntoIterator<Item = B>,
    {
        for button in buttons {
            self.buttons.push(button.into());
        }
        self
    }

    pub fn on_click(
        mut self,
        on_click: impl Fn(&NumberInputGridEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(on_click));
        self
    }
}

impl RenderOnce for NumberInputGrid {
    fn render(self, _window: &mut gpui::Window, cx: &mut gpui::App) -> impl gpui::IntoElement {
        div()
            .size_full()
            .grid()
            .grid_cols(self.cols)
            .grid_rows(self.rows)
            .gap_2()
            .children(self.buttons.into_iter().enumerate().map(|(idx, button)| {
                let Some((button_content, event)) = button.render() else {
                    return div().size_full();
                };

                div().bg(cx.theme().accent).rounded_md().size_full().child(
                    Button::new(("number-grid-button", idx))
                        .size_full()
                        .ghost()
                        .when_some(self.on_click.clone(), {
                            move |this, on_click| {
                                this.on_click(move |_, window, cx| {
                                    on_click(&event, window, cx);
                                })
                            }
                        })
                        .child(button_content),
                )
            }))
    }
}
