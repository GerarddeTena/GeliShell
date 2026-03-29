# `src/parser/` — Lexer, Tokens y AST

Este directorio convierte el **texto plano** que escribe el usuario en una **estructura de datos** (AST) que el resto de la shell puede procesar.

> 💡 No es necesario entender este módulo para usar o configurar GeliShell. Es relevante si quieres contribuir a la sintaxis o añadir soporte para nuevas construcciones (ej. subshells, here-docs, etc.).

---

## Ficheros

### `token.rs` — Tipos de tokens
**¿Qué hace?** Define los **bloques atómicos** del lenguaje de la shell. El lexer convierte caracteres en tokens.

```
"list -a | search foo > out.txt"
  │     │  │  │    │  │
  Word  Word Pipe  Word  Redirect
```

Tipos principales:
- `Word("list")` — cualquier palabra sin comillas
- `Quoted("mensaje con espacios")` — texto entre comillas
- `Variable("HOME")` — `$HOME`, `$env:PATH`, `%PATH%`
- `Pipe` — el carácter `|`
- `And` — `&&`
- `Or` — `||`
- `Semicolon` — `;`
- `Ampersand` — `&` (background)
- `Redirect { kind, target }` — `>`, `>>`, `<`

`RedirectKind` distingue entre:
- `Overwrite` → `>`
- `Append` → `>>`
- `Input` → `<`

### `lexer.rs` — Tokenizador
**¿Qué hace?** Lee el string de entrada carácter a carácter y produce una lista de `Token`.

Maneja casos especiales:
- Comillas simples `'...'` y dobles `"..."` → `Token::Quoted`
- Variables `$VAR`, `$env:VAR`, `%VAR%` → `Token::Variable`
- Operadores compuestos `&&`, `||`, `>>` — no los confunde con sus componentes simples

### `ast.rs` — Árbol Sintáctico Abstracto
**¿Qué hace?** Define las estructuras que representan comandos completos, incluyendo sus relaciones lógicas.

```rust
pub enum ASTNode {
    Command(Command),              // un solo comando: ls -la
    Pipeline(Vec<ASTNode>),        // pipe: ls | grep foo
    And(Box<ASTNode>, Box<ASTNode>),   // &&: cmd1 && cmd2
    Or(Box<ASTNode>, Box<ASTNode>),    // ||: cmd1 || cmd2
    Sequence(Box<ASTNode>, Box<ASTNode>), // ;: cmd1 ; cmd2
    Background(Box<ASTNode>),      // &: cmd &
}

pub struct Command {
    pub name: String,              // "ls", "git", "list"
    pub args: Vec<Token>,          // ["-la", "--all"]
    pub redirections: Vec<Redirection>, // [> out.txt]
}
```

### `parser.rs` — Analizador sintáctico
**¿Qué hace?** Toma la lista de `Token` del lexer y construye el `ASTNode` correcto, respetando la **precedencia** de operadores (`;` < `||` < `&&` < `|`).

**Ejemplos:**
```
"list -a"             → ASTNode::Command { name: "list", args: ["-a"] }
"list | search foo"   → ASTNode::Pipeline([Command("list"), Command("search", ["foo"])])
"build && test"       → ASTNode::And(Command("build"), Command("test"))
"update || echo fail" → ASTNode::Or(Command("update"), Command("echo", ["fail"]))
"deploy &"            → ASTNode::Background(Command("deploy"))
```

### `mod.rs`
Declara y re-exporta los submódulos. Punto de entrada del módulo `parser`.

---

## Flujo completo

```
Input: "list -a | search foo > out.txt"
          │
          ▼
        lexer.rs
          │  Word("list"), Word("-a"), Pipe, Word("search"), Word("foo"),
          │  Redirect(Overwrite, "out.txt")
          ▼
        parser.rs
          │
          ▼
     ASTNode::Pipeline([
       Command { name: "list", args: ["-a"], redirections: [] },
       Command { name: "search", args: ["foo"], redirections: [Redirect(Overwrite, "out.txt")] }
     ])
```

---

## Para contribuidores

- El parser usa **recursive descent** — cada función parsea un nivel de precedencia
- El AST viaja a `shell/guard/` (comprobación de seguridad) y luego a `shell/translator/pipeline/` (traducción)
- El único lugar donde se hace `match` sobre `ASTNode` en la lógica de traducción es `NodeDecomposer` (`shell/translator/pipeline/steps/node_decomposer.rs`)
