use crate::lexer::token::Token;

#[derive(Debug)]
pub struct CommandSlice {
    id: u32,
    name: String,
    command: Vec<Token>,
}

impl CommandSlice {
    pub fn new(id: u32, command: Vec<Token>) -> Self {
        CommandSlice {
            id,
            name: format!("Command {}", id),
            command,
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

    pub fn command(&self) -> &Vec<Token> {
        &self.command
    }
}
