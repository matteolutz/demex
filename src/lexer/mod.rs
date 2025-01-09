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

    fn peek(&self) -> char {
        self.input.chars().nth(self.position).unwrap_or('\n')
    }

    fn consume(&mut self) -> char {
        let current_char = self.peek();
        self.position += 1;
        current_char
    }

    #[allow(dead_code)]
    fn advance(&mut self) {
        self.position += 1;
    }

    fn is_eof(&self) -> bool {
        self.position >= self.input.len()
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, TokenizationError> {
        let mut tokens = Vec::<Token>::new();

        while !self.is_eof() {
            if self.peek().is_ascii_digit() {
                let mut num = String::new();

                while self.peek().is_ascii_digit() {
                    num.push(self.consume());
                }

                if self.peek() == '.' {
                    num.push(self.consume());

                    while self.peek().is_ascii_digit() {
                        num.push(self.consume());
                    }

                    tokens.push(Token::FloatingPoint(num.parse().unwrap()));
                    continue;
                }

                tokens.push(Token::Numeral(num.parse().unwrap()));
            } else if self.peek().is_alphabetic() {
                let mut keyword = String::new();

                while self.peek().is_alphabetic() {
                    keyword.push(self.consume());
                }

                if keyword == "intens" {
                    tokens.push(Token::KeywordIntens);
                } else if keyword == "thru" {
                    tokens.push(Token::KeywordThru);
                } else if keyword == "full" {
                    tokens.push(Token::KeywordFull);
                } else if keyword == "out" {
                    tokens.push(Token::KeywordOut);
                } else if keyword == "home" {
                    tokens.push(Token::KeywordHome);
                } else if keyword == "manset" {
                    tokens.push(Token::KeywordManSet);
                } else if keyword == "record" {
                    tokens.push(Token::KeywordRecord);
                } else if keyword == "group" || keyword == "g" {
                    tokens.push(Token::KeywordGroup);
                } else if keyword == "rename" {
                    tokens.push(Token::KeywordRename);
                } else if keyword == "clear" {
                    tokens.push(Token::KeywordClear);
                } else if keyword == "color" {
                    tokens.push(Token::KeywordColor);
                } else if keyword == "position" {
                    tokens.push(Token::KeywordPosition);
                } else if keyword == "preset" {
                    tokens.push(Token::KeywordPreset);
                } else if keyword == "test" {
                    tokens.push(Token::KeywordTest);
                } else {
                    return Err(TokenizationError::UnknownKeyword(keyword));
                }
            } else if self.peek() == '@' {
                self.consume();
                tokens.push(Token::KeywordIntens);
            } else if self.peek() == '+' {
                self.consume();
                tokens.push(Token::Plus);
            } else if self.peek() == '-' {
                self.consume();
                tokens.push(Token::Minus);
            } else if self.peek() == '%' {
                self.consume();
                tokens.push(Token::Percent);
            } else if self.peek() == '!' {
                self.consume();
                tokens.push(Token::Exclamation);
            } else if self.peek() == '(' {
                self.consume();
                tokens.push(Token::ParenOpen);
            } else if self.peek() == ')' {
                self.consume();
                tokens.push(Token::ParenClose);
            } else if self.peek() == '\"' {
                self.consume();

                let mut string = String::new();

                while self.peek() != '\"' {
                    if self.peek() == '\n' {
                        return Err(TokenizationError::UnterminatedString);
                    }

                    string.push(self.consume());
                }

                self.consume();

                tokens.push(Token::String(string));
            } else if self.peek() == '~' {
                self.consume();
                tokens.push(Token::KeywordFixturesSelected);
            } else if self.peek().is_whitespace() {
                self.consume();
            }
        }

        tokens.push(Token::Eof);

        Ok(tokens)
    }
}
