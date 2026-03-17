#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Word(String),
    Quoted(String),
    Variable(String),
    Redirect(RedirectKind),
    Operator(OperatorKind),
    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RedirectKind {
    Append,
    Out,
    In,
    Pipe,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OperatorKind {
    And,
    Or,
    Semicolon,
    Background,
}

impl Token {
    pub(crate) fn as_str(&self) -> Option<&str> {
        match self {
            Token::Word(s) | Token::Quoted(s) | Token::Variable(s) => Some(s.as_str()),
            _ => None,
        }
    }
}
