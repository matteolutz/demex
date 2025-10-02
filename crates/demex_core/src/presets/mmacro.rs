use serde::{Deserialize, Serialize};

use crate::{command::parser::nodes::action::Action, implement_set_property};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ui", derive(egui_probe::EguiProbe))]
pub struct MMacro {
    #[cfg_attr(feature = "ui", egui_probe(skip))]
    id: u32,

    name: String,

    #[cfg_attr(feature = "ui", egui_probe(skip))]
    action: Box<Action>,
}

impl MMacro {
    pub fn new(id: u32, name: Option<String>, action: Box<Action>) -> Self {
        MMacro {
            id,
            name: name.unwrap_or(format!("Macro {}", id)),
            action,
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn name_mut(&mut self) -> &mut String {
        &mut self.name
    }

    pub fn action(&self) -> &Action {
        &self.action
    }
}

#[derive(strum_macros::EnumString, strum_macros::Display)]
pub enum MMacroProperty {
    Name,
}

implement_set_property! {
    for MMacro with MMacroProperty,

    Name => name as String
}
