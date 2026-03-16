use crate::parser::token::{RedirectKind, Token};

/// Nodo raíz del árbol — cada variante representa una construcción
/// sintáctica distinta que el executor sabrá cómo ejecutar
#[derive(Debug, Clone, PartialEq)]
pub enum ASTNode {
    Command(Command),                     // grep -r "fooBar" ./src > out.txt
    Pipeline(Vec<ASTNode>),               // entre comandos (ls | grep <name> | wc -l)
    And(Box<ASTNode>, Box<ASTNode>),      // solo right si left true
    Or(Box<ASTNode>, Box<ASTNode>),       // right solo li left false
    Sequence(Box<ASTNode>, Box<ASTNode>), // secuencialmente right -> left
    Background(Box<ASTNode>),             // nodo interno en background
}

#[derive(Debug, Clone, PartialEq)]
pub struct Command {
    // nombre del ejecutable (grep, ls, npm, ...)
    pub name: String,
    // Args posicionales - Token no String para distinguir entre Word, Quoted o Variable
    // hasta su distincion.
    pub args: Vec<Token>,
    // Redirecciones (>, >>, <) asociadads al comando
    pub redirections: Vec<Redirection>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Redirection {
    pub kind: RedirectKind,
    // destino de la redirección (archivo o descriptor)
    pub target: Token,
}
