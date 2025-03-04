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
                    self.consume();

                    let mut fract = String::new();

                    while self.peek().is_ascii_digit() {
                        fract.push(self.consume());
                    }

                    if fract.is_empty() {
                        return Err(TokenizationError::InvalidFloatingPoint);
                    }

                    let floating_point: f32 = format!("{}.{}", num, fract).parse().unwrap();

                    let num: u32 = num.parse().unwrap();
                    let fract: u32 = fract.parse().unwrap();

                    tokens.push(Token::FloatingPoint(floating_point, (num, fract)));
                    continue;
                }

                tokens.push(Token::Integer(num.parse().unwrap()));
            } else if self.peek().is_alphabetic() {
                let mut keyword_str = String::new();

                while self.peek().is_alphabetic() {
                    keyword_str.push(self.consume());
                }

                let keyword = match keyword_str.as_str() {
                    "intens" => Some(Token::KeywordIntens),
                    "thru" => Some(Token::KeywordThru),
                    "full" => Some(Token::KeywordFull),
                    "half" => Some(Token::KeywordHalf),
                    "out" => Some(Token::KeywordOut),
                    "home" => Some(Token::KeywordHome),
                    "manset" => Some(Token::KeywordManSet),
                    "record" | "rec" => Some(Token::KeywordRecord),
                    "group" | "g" => Some(Token::KeywordGroup),
                    "macro" => Some(Token::KeywordMacro),
                    "commandslice" => Some(Token::KeywordCommandSlice),
                    "rename" | "ren" => Some(Token::KeywordRename),
                    "clear" => Some(Token::KeywordClear),
                    "color" => Some(Token::KeywordColor),
                    "position" => Some(Token::KeywordPosition),
                    "preset" => Some(Token::KeywordPreset),
                    "test" => Some(Token::KeywordTest),
                    "strobe" => Some(Token::KeywordStrobe),
                    "maintenance" => Some(Token::KeywordMaintenance),
                    "create" => Some(Token::KeywordCreate),
                    "sequence" | "seq" => Some(Token::KeywordSequence),
                    "fader" => Some(Token::KeywordFader),
                    "executor" | "exec" => Some(Token::KeywordExecutor),
                    "as" => Some(Token::KeywordAs),
                    "for" => Some(Token::KeywordFor),
                    "to" => Some(Token::KeywordTo),
                    "cue" => Some(Token::KeywordCue),
                    "with" => Some(Token::KeywordWith),
                    "all" => Some(Token::KeywordAll),
                    "active" => Some(Token::KeywordActive),
                    "update" => Some(Token::KeywordUpdate),
                    "merge" => Some(Token::KeywordMerge),
                    "override" => Some(Token::KeywordOverride),
                    "nuzul" => Some(Token::KeywordNuzul),
                    "sueud" => Some(Token::KeywordSueud),
                    "save" => Some(Token::KeywordSave),
                    "delete" | "del" => Some(Token::KeywordDelete),
                    "really" => Some(Token::KeywordReally),
                    "next" => Some(Token::KeywordNext),
                    "config" => Some(Token::KeywordConfig),
                    "output" => Some(Token::KeywordOutput),
                    "beam" => Some(Token::KeywordBeam),
                    "control" => Some(Token::KeywordControl),
                    "focus" => Some(Token::KeywordFocus),
                    "feature" => Some(Token::KeywordFeature),
                    "assign" => Some(Token::KeywordAssign),
                    "unassign" => Some(Token::KeywordUnassign),
                    "go" => Some(Token::KeywordGo),
                    "stop" => Some(Token::KeywordStop),
                    "flash" => Some(Token::KeywordFlash),
                    "effect" => Some(Token::KeywordEffect),
                    "button" => Some(Token::KeywordButton),
                    _ => None,
                };

                if let Some(keyword) = keyword {
                    tokens.push(keyword);
                } else {
                    return Err(TokenizationError::UnknownKeyword(keyword_str));
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
            } else if self.peek() == ',' {
                self.consume();
                tokens.push(Token::Comma);
            } else if self.peek().is_whitespace() {
                self.consume();
            } else {
                return Err(TokenizationError::UnknownCharacter(self.peek()));
            }
        }

        tokens.push(Token::Eof);

        Ok(tokens)
    }
}
