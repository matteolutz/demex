use serde::{Deserialize, Serialize};

use crate::parser::nodes::action::Action;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MMacro {
    id: u32,
    name: String,
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
