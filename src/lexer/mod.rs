use self::{error::TokenizationError, token::Token};

pub mod error;
pub mod token;

pub struct Lexer<'a> {
    input: &'a str,
    position: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Lexer<'a> {
        Lexer { input, position: 0 }
    }

    pub fn peek(&self) -> char {
        self.input.chars().nth(self.position).unwrap()
    }

    pub fn consume(&mut self) -> char {
        let current_char = self.peek();
        self.position += 1;
        return current_char;
    }

    pub fn advance(&mut self) {
        self.position += 1;
    }

    pub fn is_eof(&self) -> bool {
        self.position >= self.input.len() - 1
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, TokenizationError> {
        let mut tokens = Vec::<Token>::new();

        while !self.is_eof() {
            if self.peek().is_digit(10) {
                let mut num = String::new();

                while self.peek().is_digit(10) {
                    num.push(self.consume());
                }

                tokens.push(Token::Numeral(num.parse().unwrap()));
            } else if self.peek().is_alphabetic() {
                let mut keyword = String::new();

                while self.peek().is_alphabetic() {
                    keyword.push(self.consume());
                }

                if keyword == "intens" {
                    tokens.push(Token::Intens);
                } else {
                    return Err(TokenizationError::UnknownKeyword(keyword));
                }
            } else if self.peek().is_whitespace() {
                self.consume();
            }
        }

        return Ok(tokens);
    }
}
