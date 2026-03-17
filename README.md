# GeliShell

Shell interactiva cross-platform escrita en Rust que:

- Traduce comandos canónicos a Bash, Zsh, Fish, PowerShell o CMD.
- Aplica reglas de seguridad antes de ejecutar.
- Ejecuta comandos nativos con streaming y control de timeout.
- Incluye asistente local con recuperación de contexto (RAG).

---

## ¿Qué es GeliShell?

GeliShell es una REPL con pipeline semántico:

`input -> lexer -> parser(AST) -> builtins -> guard -> translator -> executor`

Su objetivo es permitir una capa de comandos “canónicos” (ej. `list`, `copy`, `find`) y traducirlos al subsistema activo de forma consistente.

---

## Características principales

- Traductor de comandos canónicos con mapa en `src/commands/commands.toml`.
- Soporte multi-subsistema: Bash, Zsh, Fish, PowerShell y CMD.
- Guardrails de seguridad para operaciones destructivas y patrones peligrosos.
- Builtins de sesión: `cd`, `clear`, `exit`, `export`, `unset`, `history`, `source`, `g`.
- Historial persistente de comandos y navegación inteligente tipo z/fzf (`g jump`).
- UI interactiva: help menu, config menu y assistant menu.
- RAG local sobre `docs.db` con `sqlite-vec`.

---

## Arquitectura de alto nivel

### Flujo de ejecución

1. **Entrada REPL** (`read_repl_input`) con historial + sugerencias.
2. **Lexer** (`src/parser/lexer.rs`) tokeniza (límite 64KB).
3. **Parser** (`src/parser/parser.rs`) construye AST.
4. **Builtins** (`src/shell/builtins`) se ejecutan antes del traductor.
5. **Guard** (`src/shell/guard`) valida riesgos semánticos.
6. **TranslationPipeline** (`src/shell/translator/pipeline`) transforma AST a comando nativo.
7. **Executor** (`src/shell/executor`) ejecuta de forma async con streaming stdout/stderr.

### Pipeline de traducción (steps)

1. `NodeDecomposer` (AST -> `Vec<CommandFragment>`)
2. `CommandResolver` (lookup canónico en `CommandMap`)
3. `FlagResolver` (flags canónicos -> flags nativos)
4. `VariableExpander` (`$VAR` -> sintaxis de subsistema)
5. `SubsystemMapper` (resolución final por subsistema)

---

## Módulos clave

| Módulo | Ruta | Responsabilidad |
| --- | --- | --- |
| Parser | `src/parser/*` | Lexer + Parser + AST |
| Translator | `src/shell/translator/*` | Resolución canónica + mapping por subsistema |
| Guard | `src/shell/guard/*` | Bloqueo/confirmación de comandos peligrosos |
| Executor | `src/shell/executor/*` | Spawn async + streaming + timeout |
| Builtins | `src/shell/builtins/*` | Comandos internos de sesión |
| TUI | `src/shell/tui/*` | Menús interactivos y lectura avanzada de input |
| Assistant | `src/shell/assistant/*` | Bootstrap de modelo + RAG + sugerencias |
| Config | `src/shell/config/*` | Carga/guardado config, bootstrap de runtime e historial |

---

## Comandos integrados (Builtins)

| Comando | Descripción |
| --- | --- |
| `cd <ruta>` | Cambia directorio y actualiza `PWD` / `OLDPWD` |
| `clear` | Limpia pantalla + scrollback |
| `exit [code]` | Termina la shell |
| `export K=V` | Define variables de entorno de sesión |
| `unset K` | Elimina variables de entorno |
| `history` / `history --clear` | Muestra o limpia historial de comandos |
| `g` / `g <pattern>` / `g -` / `g --clear` | Navegación inteligente por frecency |
| `source <file>` | Stub actual (motor de scripting pendiente) |

---

## Triggers y atajos

### Triggers escritos

- `geli-helpme` -> abre Help Menu.
- `geli-config-me` -> abre Config Menu.
- `gerisabet` -> abre Assistant Menu.
- `:stop` / `:stop*` -> intercepta stop de comando en ejecución.
- `:search` / `:search*` -> shortcut para búsqueda interactiva (skeleton actual).

### Teclado (REPL)

- `Ctrl+D`: salir.
- `Ctrl+H` o `Ctrl+?`: help.
- `Ctrl+L`: clear.
- `Ctrl+Alt+S`: config menu.
- `Ctrl+Alt+G`: assistant menu.
- `Ctrl+S`: search action.
- `Tab` / `Right`: autocomplete “ghost”.
- `Up` / `Down`: historial.

---

## Mapa de comandos canónicos

Archivo fuente: `src/commands/commands.toml`

- ~39 comandos canónicos.
- Categorías: `filesystem`, `file-ops`, `process`, `network`, `text`, `system`, `dev`.
- Traducciones por subsistema + sugerencias + flags canónicos.

Ejemplo de estructura:

```toml
[[commands]]
name = "list"
category = "filesystem"
translate = {
  bash = { exact = "ls", suggestions = ["ls -la"] },
  powershell = { exact = "Get-ChildItem", suggestions = ["gci", "dir"] },
  cmd = { exact = "dir", suggestions = ["dir /b"] }
}

[[commands.flags]]
canonical = "--recursive"
bash = "-R"
powershell = "-Recurse"
cmd = "/s"
```

---

## Seguridad (Guardrails)

Reglas activas (`default_guard()`):

- `RmGuard`: bloquea `rm` recursivo+forzado hacia raíces críticas.
- `ChmodChownGuard`: bloquea `chmod/chown -R` en rutas protegidas.
- `DdGuard`: bloquea escrituras `dd of=/dev/...` a block devices.
- `MkfsGuard`: requiere confirmación explícita (`--yes-i-know-what-i-am-doing`).
- `CriticalRedirectGuard`: bloquea redirecciones a archivos críticos (`/etc/passwd`, `/etc/shadow`, etc.).
- `PipeExecutionGuard`: bloquea patrones `curl|bash` / `wget|sh`.
- `ForkBombGuard`: detecta patrón de fork bomb en AST.

---

## Assistant y RAG local

Componentes:

- `src/shell/assistant/qwen.rs`: bootstrap y gestión de artefacto GGUF.
- `src/shell/assistant/rag.rs`: retrieval semántico en `docs.db` vía `sqlite-vec`.
- `src/shell/assistant/params.rs`: menú de prompts predefinidos.
- `src/shell/assistant/suggest.rs`: composición de prompt/salida.

Variables de entorno relevantes:

- `GELI_DOCS_DB_PATH`
- `GELI_SQLITE_VEC_PATH`
- `GELI_EMBED_MODEL` (default `nomic-embed-text`)
- `GELI_OLLAMA_URL` (default `http://127.0.0.1:11434`)

Estado actual:

- El retrieval RAG vectorial está integrado.
- La generación de respuesta del asistente actualmente usa una síntesis local por plantillas (no inferencia GGUF completa end-to-end en este estado del código).

---

## How-to rápido (lo más relevante)

### 1) Ejecutar GeliShell

```powershell
cargo run
```

Alternativa binario compilado:

```powershell
.\target\debug\geli_shell.exe
```

### 2) Forzar subsistema de traducción

Temporal por entorno:

```powershell
$env:GELI_SUBSYSTEM = "powershell"
```

Persistente en config (`~/.config/geliShell/config.toml`):

```toml
[subsystem]
override_subsystem = "powershell"
```

### 3) Personalizar comandos propios

En `config.toml`:

```toml
[[customization.custom_commands]]
name = "ll"
template = "list --all --long"
```

Luego en REPL:

```text
ll
```

### 4) Usar navegación inteligente `g`

```text
g           # muestra top de rutas aprendidas
g rust      # salta al mejor match por frecency
g -         # vuelve a OLDPWD
g --clear   # limpia historial de g
```

### 5) Usar menús interactivos

```text
geli-helpme
geli-config-me
gerisabet
```

### 6) Reconstruir Knowledge Base RAG (incluye docs + scripting KB)

Script recomendado en raíz:

```powershell
.\rebuild-rag.ps1
```

Con parámetros:

```powershell
.\rebuild-rag.ps1 `
  -DocsDbPath "$env:USERPROFILE\.config\geliShell\models\docs.db" `
  -VecDllPath "$env:USERPROFILE\.config\geliShell\models\vec0.dll" `
  -BatchSize 16 `
  -Model "nomic-embed-text" `
  -OllamaUrl "http://127.0.0.1:11434"
```

Script low-level equivalente:

```powershell
cargo run --bin build_docs_db -- --help
```

---

## Estructura del repositorio

```text
src/
  main.rs                       # loop REPL y orquestación
  parser/                       # lexer, parser, AST, tokens
  shell/
    builtins/                   # cd, clear, export, unset, history, source, g
    guard/                      # reglas de seguridad
    translator/                 # map canónico + pipeline + resolver
    executor/                   # ejecución async
    config/                     # config, bootstrap, historial
    tui/                        # help/config/assistant menus + input
    assistant/                  # qwen/rag/suggest/params
  commands/commands.toml        # mapa de traducción
docs/
  kb/                           # documentación base para RAG
scripting-*-rag.md             # KB operativa (básico/medio/avanzado)
rebuild-rag.ps1                # rebuild automático de docs.db
```

---

## Desarrollo

Comandos recomendados:

```powershell
cargo check
cargo test
cargo run
```

`Cargo.toml` usa Rust 2024 y dependencias clave como `tokio`, `crossterm`, `rusqlite`, `reqwest`, `serde`, `thiserror`.

---

## Limitaciones actuales (estado del repo)

- `source` builtin es un stub (pendiente de motor de scripting).
- El selector modal existe (`src/shell/selector`) pero su acople total al flujo principal aún está incompleto.
- El asistente está orientado a plantillas seguras + RAG; la inferencia GGUF completa aún no está cerrada en esta versión.

---

## Rutas de runtime

- Config: `~/.config/geliShell/config.toml`
- Historial REPL: `~/.config/geliShell/history.txt`
- Historial `g`: `~/.config/geliShell/g_history.toml`
- Modelos/docs assistant: `~/.config/geliShell/models/`
  - `docs.db`
  - `vec0.dll` (o `.so`/`.dylib` por plataforma)

En Windows, el root de config usa `%USERPROFILE%\.config\geliShell`.
