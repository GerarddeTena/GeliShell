use crate::parser::ast::{ASTNode, Command, Redirection};
use crate::parser::token::{OperatorKind, RedirectKind, Token};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("unexpected token: {0:?}")]
    UnexpectedToken(Token),
    #[error("unexpected end of file")]
    UnexpectedEof,
    #[error("missing target for redirect")]
    MissingRedirectTarget,
}

pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    pub fn new(mut tokens: Vec<Token>) -> Self {
        // Guarantee there is always at least an Eof sentinel so peek()
        // and advance() never index out-of-bounds on an empty token stream.
        if tokens.is_empty() || !matches!(tokens.last(), Some(Token::Eof)) {
            tokens.push(Token::Eof);
        }
        Parser {
            tokens,
            position: 0,
        }
    }

    // Consuming — igual que el Lexer, un parser usado no tiene valor
    pub fn parse(mut self) -> Result<ASTNode, ParseError> {
        let node = self.parse_sequence()?;
        // Al terminar debe haber consumido todo hasta Eof
        match self.peek() {
            Token::Eof => Ok(node),
            t => Err(ParseError::UnexpectedToken(t.clone())),
        }
    }

    // ----------------------------------------------------------
    // Nivel 1 — secuencias: cmd && cmd  |  cmd ; cmd  |  cmd || cmd
    // ----------------------------------------------------------
    fn parse_sequence(&mut self) -> Result<ASTNode, ParseError> {
        let mut left = self.parse_pipeline()?;

        loop {
            match self.peek() {
                Token::Operator(OperatorKind::And) => {
                    self.advance();
                    let right = self.parse_pipeline()?;
                    left = ASTNode::And(Box::new(left), Box::new(right));
                }
                Token::Operator(OperatorKind::Or) => {
                    self.advance();
                    let right = self.parse_pipeline()?;
                    left = ASTNode::Or(Box::new(left), Box::new(right));
                }
                Token::Operator(OperatorKind::Semicolon) => {
                    self.advance();
                    // ; al final de input es válido — "cmd ;" es legal en bash
                    if matches!(self.peek(), Token::Eof) {
                        break;
                    }
                    let right = self.parse_pipeline()?;
                    left = ASTNode::Sequence(Box::new(left), Box::new(right));
                }
                _ => break,
            }
        }
        Ok(left)
    }

    // ----------------------------------------------------------
    // Nivel 2 — pipelines: cmd | cmd | cmd
    // ----------------------------------------------------------
    fn parse_pipeline(&mut self) -> Result<ASTNode, ParseError> {
        let mut commands = vec![self.parse_command()?];

        while matches!(self.peek(), Token::Redirect(RedirectKind::Pipe)) {
            self.advance(); // consume '|'
            commands.push(self.parse_command()?);
        }

        if commands.len() == 1 {
            // No era un pipeline — devuelve el comando directamente
            Ok(commands.remove(0))
        } else {
            Ok(ASTNode::Pipeline(commands))
        }
    }

    // ----------------------------------------------------------
    // Nivel 3 — comando simple: nombre + args + redirections
    // ----------------------------------------------------------
    fn parse_command(&mut self) -> Result<ASTNode, ParseError> {
        // El primer token debe ser el nombre del comando
        let name = match self.advance_if_word() {
            Some(w) => w,
            None => return Err(ParseError::UnexpectedToken(self.peek().clone())),
        };

        let mut args = Vec::new();
        let mut redirections = Vec::new();

        loop {
            match self.peek() {
                // Más argumentos
                Token::Word(_) | Token::Quoted(_) | Token::Variable(_) => {
                    args.push(self.advance().clone());
                }
                // Redirección: consume el operador y espera un Word/Quoted/Variable destino
                Token::Redirect(kind) if *kind != RedirectKind::Pipe => {
                    let kind = kind.clone();
                    self.advance();
                    match self.peek() {
                        Token::Word(_) | Token::Quoted(_) | Token::Variable(_) => {
                            let target = self.advance().clone();
                            redirections.push(Redirection { kind, target });
                        }
                        Token::Eof => return Err(ParseError::MissingRedirectTarget),
                        t => return Err(ParseError::UnexpectedToken(t.clone())),
                    }
                }
                // Background: & al final del comando
                Token::Operator(OperatorKind::Background) => {
                    self.advance();
                    return Ok(ASTNode::Background(Box::new(ASTNode::Command(Command {
                        name,
                        args,
                        redirections,
                    }))));
                }
                _ => break,
            }
        }

        Ok(ASTNode::Command(Command {
            name,
            args,
            redirections,
        }))
    }

    // ----------------------------------------------------------
    // Helpers
    // ----------------------------------------------------------

    fn peek(&self) -> &Token {
        &self.tokens[self.position]
    }

    fn advance(&mut self) -> &Token {
        let t = &self.tokens[self.position];
        if self.position < self.tokens.len() - 1 {
            self.position += 1;
        }
        t
    }

    // Avanza solo si el token actual es Word — sin consumir si no lo es
    fn advance_if_word(&mut self) -> Option<String> {
        match self.peek() {
            Token::Word(w) => {
                let w = w.clone();
                self.advance();
                Some(w)
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::lexer::Lexer;

    #[test]
    fn parses_redirection_targets_with_quotes_and_variables() {
        let tokens = Lexer::new("search foo > \"out file.txt\" < $INPUT")
            .tokenize()
            .unwrap();
        let ast = Parser::new(tokens).parse().unwrap();

        let ASTNode::Command(command) = ast else {
            panic!("expected command node");
        };

        assert_eq!(command.redirections.len(), 2);
        assert!(matches!(command.redirections[0].target, Token::Quoted(_)));
        assert!(matches!(command.redirections[1].target, Token::Variable(_)));
    }
}
