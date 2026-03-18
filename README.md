# GeliShell
![Rust](https://img.shields.io/badge/rust-edition%202024-orange.svg)
![License](https://img.shields.io/badge/license-MIT-blue.svg)

Shell interactiva cross-platform escrita en Rust.

GeliShell ofrece una capa de comandos canónicos (por ejemplo `list`, `copy`, `find`) y los traduce al subsistema activo (`bash`, `zsh`, `fish`, `powershell`, `cmd`) antes de ejecutar.

## Qué hace hoy

- Traduce comandos canónicos con mapa en `src/commands/commands.toml`.
- Aplica guardrails semánticos antes de ejecutar.
- Ejecuta comandos nativos de forma asíncrona con streaming en tiempo real y timeout opcional.
- Incluye builtins de sesión: `cd`, `clear`, `exit`, `export`, `unset`, `history`, `source`, `g`.
- Incluye historial persistente del REPL y navegación por frecencia (`g jump`).
- Incluye asistente local con recuperación RAG sobre `docs.db` y catálogo interactivo `gerisabet --show-me`.

## ¿Por qué GeliShell? (Filosofía)
A diferencia de las shells tradicionales que te obligan a aprender una sintaxis nueva o limitan tu entorno, GeliShell actúa como un **metacompilador interactivo de comandos**.
- **Escribe una vez, ejecuta en cualquier parte:** Aprende los comandos canónicos de GeliShell y úsalos indistintamente en Windows, Linux o macOS. La shell se encarga de traducirlos al subsistema subyacente.
- **Seguridad por diseño:** El Guardrail semántico evita desastres (como un borrado recursivo accidental) interceptando el AST antes de que el sistema operativo lo vea.
- **IA Integrada y Determinista:** No es un simple wrapper de ChatGPT. Es un sistema RAG local, ultrarrápido y confinado a la documentación técnica, diseñado para ser útil sin ser peligroso.

## Requisitos Previos (Prerequisites)
Para compilar y ejecutar GeliShell desde el código fuente, necesitas:
- **Rust Toolchain:** Edición 2024 (1.85+ recomendado).
- **Para el Asistente RAG (Opcional pero recomendado):**
  - Motor Ollama ejecutándose localmente (`ollama serve`).
  - Modelo de embeddings: `ollama pull nomic-embed-text`.
  - Extensión `sqlite-vec` descargada y apuntada por la variable `$env:GELI_SQLITE_VEC_PATH`.
---

## Flujo de ejecución real

Pipeline principal en `src/main.rs`:

`input -> lexer -> parser(AST) -> builtins -> guard -> translator -> executor`

Detalle por etapa:

1. **Entrada REPL** (`read_repl_input`) con historial y sugerencias tipo ghost.
2. **Lexer** (`src/parser/lexer.rs`) tokeniza con límite de 64KB.
3. **Parser** (`src/parser/parser.rs`) construye el AST.
4. **Builtins** (`BuiltinRegistry::try_execute`) se evalúan antes de traducir.
5. **Guard** (`default_guard()`) bloquea o exige confirmación semántica.
6. **TranslationPipeline** (`src/shell/translator/pipeline`) produce un comando nativo string.
7. **Executor** (`src/shell/executor`) ejecuta async con `tokio::process::Command`.

### Steps del traductor

1. `NodeDecomposer` (`ASTNode -> Vec<CommandFragment>`)
2. `CommandResolver`
3. `FlagResolver`
4. `VariableExpander`
5. `SubsystemMapper`

---

## Arquitectura por módulos

| Módulo | Ruta | Responsabilidad |
| --- | --- | --- |
| Parser | `src/parser/*` | Lexer, parser, AST, tokens |
| Translator | `src/shell/translator/*` | Mapa canónico, resolución, pipeline por subsistema |
| Guard | `src/shell/guard/*` | Reglas de seguridad sobre AST |
| Executor | `src/shell/executor/*` | Spawn async, streaming stdout/stderr, timeout |
| Builtins | `src/shell/builtins/*` | Comandos internos de sesión |
| Config | `src/shell/config/*` | Bootstrap runtime, config, historial persistente |
| TUI | `src/shell/tui/*` | Menús interactivos y captura avanzada de input |
| Assistant | `src/shell/assistant/*` | Bootstrap de modelo, RAG, sugerencias |
| Selector | `src/shell/selector/*` | Selector modal de alternativas (actualmente desacoplado del flujo final) |

---

## Builtins disponibles

| Comando | Comportamiento actual |
| --- | --- |
| `cd <ruta>` | Cambia directorio y actualiza `PWD`/`OLDPWD` |
| `clear` | Limpia pantalla/buffer |
| `exit [code]` | Termina la shell |
| `export K=V` | Define variable de entorno de sesión |
| `unset K` | Elimina variable de entorno |
| `history` / `history --clear` | Muestra o limpia historial de sesión en memoria |
| `g` / `g <pattern>` / `g -` / `g --clear` | Navegación inteligente por frecencia |
| `source <file>` | Stub: reporta que el motor de scripting aún no está disponible |

---

## Atajos y triggers

### Triggers escritos

- `geli-helpme` abre Help Menu.
- `geli-config-me` abre Config Menu.
- `gerisabet` abre Assistant Menu.
- `gerisabet --how-to "<consulta>"` solicita recomendación ejecutable con confirmación.
- `gerisabet --show-me` abre el catálogo TUI de comandos RAG (categorías + tabla filtrada por subsistema).
- `:stop` / `:stop*` interceptan stop.
- `:search` / `:search*` interceptan search (UI de búsqueda aún skeleton).

### Teclado REPL

- `Ctrl + D`: salir.
- `Ctrl + H` o `Ctrl + ?`: help.
- `Ctrl + L`: clear.
- `Ctrl + Alt + S`: config menu.
- `Ctrl + Alt + G`: assistant menu.
- `Ctrl + S`: acción search.
- `Tab` o `Right`: autocompletado ghost.
- `Up` / `Down`: historial.

---

## Mapa de comandos canónicos

Fuente: `src/commands/commands.toml`

Estado actual del archivo:

- 20 comandos canónicos cargados (`[[commands]]`).
- Categorías activas: `filesystem`, `file-ops`, `process`, `network`, `text`, `system`, `dev`.
- Traducción por subsistema con `exact` y `suggestions`.
- Flags canónicos por comando en `[[commands.flags]]`.

Estructura usada:

```toml
[[commands]]
name = "list"
description = "List directory contents"
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

Reglas activas en `default_guard()`:

- `RmGuard`
- `ChmodChownGuard`
- `DdGuard`
- `MkfsGuard`
- `CriticalRedirectGuard`
- `PipeExecutionGuard`
- `ForkBombGuard`

Tipos de error relevantes (`src/shell/guard/error.rs`):

- `DestructiveFs`
- `DiskDestroyer`
- `CriticalRedirect`
- `PipeExecution`
- `ForkBomb`
- `RequiresConfirmation`
- `BlacklistedCommand`
- `ForbiddenArgument`

---
## Interfaz de Usuario Avanzada (TUI)
GeliShell abandona la rigidez del prompt tradicional en favor de una experiencia inmersiva utilizando `crossterm`:
- **Máquinas de Estado Modales:** Navegación por menús complejos sin ensuciar el historial de la terminal (Alternate Screen).
- **Catálogo Interactivo (`--show-me`):** Explora y ejecuta comandos del RAG desde una máquina de estados (`CategoryList` y `CommandTable`) construida dinámicamente desde SQLite, sin hardcodeo de categorías/operaciones.
- **Filtrado por subsistema + placeholders:** En Estado B filtra por subsistema activo cuando aplica (con fallback visible) y resuelve parámetros `<marcador>` antes de confirmar ejecución.
---
## Assistant y RAG local

Componentes:

- `src/shell/assistant/qwen.rs`: gestión de artefactos GGUF y runtime del asistente.
- `src/shell/assistant/rag.rs`: retrieval semántico en SQLite (`docs.db`) con `sqlite-vec`.
- `src/shell/assistant/params.rs`: parámetros predefinidos del menú.
- `src/shell/assistant/suggest.rs`: prompts y parseo de salida (`--how-to`).

Variables de entorno utilizadas:

- `GELI_DOCS_DB_PATH`
- `GELI_SQLITE_VEC_PATH`
- `GELI_EMBED_MODEL` (default: `nomic-embed-text`)
- `GELI_OLLAMA_URL` (default: `http://127.0.0.1:11434`)
> **IMPORTANTE**
> - El retrieval RAG está integrado.
> - La respuesta del asistente se sintetiza localmente a partir del contexto recuperado.
> - La base RAG no se incluye en el repo: usa `rebuild-rag.ps1` para generarla localmente.
---

## Configuración y rutas de runtime

`ShellConfig` se persiste en `config.toml` con bloques:

- `[behavior]` (`selector_mode`: `always` | `auto` | `once`)
- `[subsystem]` (`override_subsystem`)
- `[execution]` (captura/timeout)
- `[visual]` (colores ANSI + tipografía)
- `[customization]` (comandos personalizados)
- `[assistant]` (modelo, `rag_top_k`, auto-unload)

Rutas canónicas:

- Config: `~/.config/geliShell/config.toml`
- Historial REPL: `~/.config/geliShell/history.txt`
- Historial de `g`: `~/.config/geliShell/g_history.toml`
- Docs RAG: `~/.config/geliShell/docs/docs.db`
- Modelos/assistant: `~/.config/geliShell/models/`
  - `qwen/*.gguf` (según variante configurada)
  - `vec0.dll` (Windows) / variante de plataforma

Resolución del root:
- Windows: `%USERPROFILE%\.config\geliShell`
- Unix: `$HOME/.config/geliShell`

Override opcional:
- `GELI_DOCS_DB_PATH` apunta a una ruta explícita y tiene prioridad sobre la ruta canónica.

---

## Uso rápido

### Ejecutar la shell

```powershell
cargo run
```

O binario ya compilado:

```powershell
.\target\debug\geli_shell.exe
```

### Forzar subsistema

Temporal por entorno:

```powershell
$env:GELI_SUBSYSTEM = "powershell"
```

Persistente en config:

```toml
[subsystem]
override_subsystem = "powershell"
```

### Comandos personalizados

```toml
[[customization.custom_commands]]
name = "ll"
template = "list --all --long"
```

### Uso de `g`

```text
g
g rust
g -
g --clear
```

### Rebuild de base RAG

Script helper:

```powershell
.\rebuild-rag.ps1
```

Genera/actualiza `docs.db` en `~/.config/geliShell/docs/docs.db` (o en la ruta indicada por `-DocsDbPath`).

CLI low-level:

```powershell
cargo run --bin build_docs_db -- --help
```

---

## Estructura del repositorio

```text
src/
  main.rs
  lib.rs
  parser/
  shell/
    assistant/
    banner.rs
    builtins/
    config/
    executor/
    guard/
    reporter.rs
    selector/
    translator/
    tui/
      show_me/
  commands/commands.toml
  bin/build_docs_db.rs
docs/kb/
rebuild-rag.ps1
```

---

## Desarrollo

Comandos recomendados:

```powershell
cargo check
cargo test
cargo run
```

El proyecto usa Rust 2024 (`edition = "2024"`) y crates como `tokio`, `crossterm`, `rusqlite`, `reqwest`, `serde`, `thiserror`.

---

## Limitaciones y estado actual

- `source` builtin sigue en modo stub.
- El selector modal existe (`src/shell/selector`), pero `SelectorMode` todavía no altera la ejecución final en `src/main.rs`.
- La acción `:search` está interceptada, pero la UI de búsqueda avanzada sigue en estado skeleton.
- El asistente depende de contexto RAG y síntesis local; no está en modo LLM completo de generación libre.
