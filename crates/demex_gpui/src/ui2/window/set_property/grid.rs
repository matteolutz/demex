use crate::ui2::{
    components::number_input_grid::NumberInputGridButton, window::set_property::NumberInputMode,
};

pub trait NumberInputModeExt {
    fn get_number_grid_buttons(self) -> Option<Vec<NumberInputGridButton>>;
}

impl NumberInputModeExt for NumberInputMode {
    fn get_number_grid_buttons(self) -> Option<Vec<NumberInputGridButton>> {
        let (allow_negative, is_float) = match self {
            Self::Float { allow_negative } => (allow_negative, true),
            Self::Integer { allow_negative } => (allow_negative, false),
            Self::None => return None,
        };

        let mut buttons = "1234567890"
            .chars()
            .into_iter()
            .map(NumberInputGridButton::insert_char)
            .collect::<Vec<_>>();

        // insert between 9 and 0
        buttons.insert(
            buttons.len() - 1,
            if allow_negative {
                NumberInputGridButton::insert("-")
            } else {
                NumberInputGridButton::Empty
            },
        );

        buttons.extend([
            NumberInputGridButton::Empty,
            // new row
            NumberInputGridButton::Delete,
            if is_float {
                NumberInputGridButton::insert(".")
            } else {
                NumberInputGridButton::Empty
            },
            NumberInputGridButton::Submit,
        ]);

        Some(buttons)
    }
}
