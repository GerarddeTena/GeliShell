// Convertir &str en Vec<String> sin interpretar significado alguno.

use crate::parser::token::{OperatorKind, RedirectKind, Token};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LexError {
    #[error("unexpected character '{0}' at position {1}")]
    UnexpectedChar(char, usize),
    #[error("unterminated quote starting at position {0}")]
    UnterminatedQuote(usize),
    #[error("input too long: {0} bytes (max {1})")]
    InputTooLong(usize, usize),
}

pub struct Lexer<'a> {
    input: &'a str,
    chars: Vec<char>,
    position: usize,
}

const MAX_INPUT_BYTES: usize = 65_536; // 64KB

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Lexer {
            input,
            chars: input.chars().collect(),
            position: 0,
        }
    }

    pub fn tokenize(mut self) -> Result<Vec<Token>, LexError> {
        if self.input.len() > MAX_INPUT_BYTES {
            return Err(LexError::InputTooLong(self.input.len(), MAX_INPUT_BYTES));
        }
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token()?;
            let is_eof = token == Token::Eof;
            tokens.push(token);
            if is_eof {
                break;
            }
        }
        Ok(tokens)
    }

    fn next_token(&mut self) -> Result<Token, LexError> {
        self.skip_whitespace();

        if self.position >= self.chars.len() {
            return Ok(Token::Eof);
        }

        match self.peek() {
            '"' | '\'' => self.read_quoted(),
            '$' => self.read_variable(),
            '>' | '<' | '|' | '&' | ';' => self.read_operator(),
            c if c.is_alphanumeric() || matches!(c, '-' | '.' | '/' | '_' | '~') => {
                self.read_word()
            }
            c => Err(LexError::UnexpectedChar(c, self.position)),
        }
    }

    fn peek(&self) -> char {
        self.chars[self.position]
    }

    fn advance(&mut self) -> char {
        let c = self.chars[self.position];
        self.position += 1;
        c
    }

    fn skip_whitespace(&mut self) {
        while self.position < self.chars.len() && self.chars[self.position].is_whitespace() {
            self.position += 1;
        }
    }

    fn read_word(&mut self) -> Result<Token, LexError> {
        let mut word = String::new();
        while self.position < self.chars.len() {
            let c = self.peek();
            // Un word termina al encontrar espacio u operador
            if c.is_whitespace() || matches!(c, '>' | '<' | '|' | '&' | ';' | '"' | '\'') {
                break;
            }
            word.push(self.advance());
        }
        Ok(Token::Word(word))
    }

    fn read_quoted(&mut self) -> Result<Token, LexError> {
        let quote_char = self.advance();
        let start = self.position;
        let mut content = String::new();

        loop {
            if self.position >= self.chars.len() {
                return Err(LexError::UnterminatedQuote(start));
            }
            let c = self.advance();
            if c == quote_char {
                break;
            }
            content.push(c);
        }
        Ok(Token::Quoted(content))
    }

    fn read_variable(&mut self) -> Result<Token, LexError> {
        self.advance(); // consume '$'
        let mut name = String::new();
        while self.position < self.chars.len() {
            let c = self.peek();
            if c.is_alphanumeric() || c == '_' {
                name.push(self.advance());
            } else {
                break;
            }
        }
        Ok(Token::Variable(name))
    }
    fn read_operator(&mut self) -> Result<Token, LexError> {
        let c = self.advance();
        let token = match c {
            '|' => Token::Redirect(RedirectKind::Pipe),
            '<' => Token::Redirect(RedirectKind::In),
            '>' => {
                // distingue > de >>
                if self.position < self.chars.len() && self.peek() == '>' {
                    self.advance();
                    Token::Redirect(RedirectKind::Append)
                } else {
                    Token::Redirect(RedirectKind::Out)
                }
            }
            '&' => {
                // distingue & de &&
                if self.position < self.chars.len() && self.peek() == '&' {
                    self.advance();
                    Token::Operator(OperatorKind::And)
                } else {
                    Token::Operator(OperatorKind::Background)
                }
            }
            ';' => Token::Operator(OperatorKind::Semicolon),
            _ => unreachable!(), // next_token ya filtró los chars válidos
        };
        Ok(token)
    }
}
