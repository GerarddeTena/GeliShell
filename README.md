# GeliShell
![Rust](https://img.shields.io/badge/rust-edition%202024-orange.svg)
![License](https://img.shields.io/badge/license-Apache_2.0-green.svg)
![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20Linux%20%7C%20macOS-lightgrey.svg)

Shell interactiva cross-platform escrita en Rust. Dos binarios, instalación en un paso.

| Binario | Función |
|---------|---------|
| `geli` | Shell REPL interactiva con traducción canónica, guardrails y historial |
| `gerisabet` | Asistente IA local con RAG sobre `docs.db` y catálogo TUI interactivo |

GeliShell traduce comandos canónicos (`list`, `copy`, `find`…) al subsistema activo (`bash`, `zsh`, `fish`, `powershell`, `cmd`) antes de ejecutar. Nunca necesitas recordar si es `ls` o `Get-ChildItem`.

## Qué hace hoy

- Traduce comandos canónicos con mapa TOML (runtime con fallback embebido en binario).
- Aplica guardrails semánticos sobre el AST antes de ejecutar.
- Ejecuta comandos nativos de forma asíncrona con streaming en tiempo real y timeout opcional.
- Builtins de sesión: `cd`, `clear`, `exit`, `export`, `unset`, `history`, `source`, `g`, `gerisabet`.
- Historial persistente del REPL y navegación frecency-based (`g jump`).
- Selector modal de alternativas de traducción según `SelectorMode`.
- Asistente local con recuperación RAG sobre `docs.db`, catálogo interactivo `--show-me` y consultas `--how-to`.

## ¿Por qué GeliShell? (Filosofía)

A diferencia de las shells tradicionales que te obligan a aprender una sintaxis nueva o limitan tu entorno, GeliShell actúa como un **metacompilador interactivo de comandos**.

- **Escribe una vez, ejecuta en cualquier parte:** Aprende los comandos canónicos de GeliShell y úsalos indistintamente en Windows, Linux o macOS. La shell se encarga de traducirlos al subsistema subyacente.
- **Seguridad por diseño:** El guardrail semántico evita desastres (borrado recursivo accidental, fork bombs, pipe-executions de red) interceptando el AST antes de que el OS lo vea.
- **IA integrada y determinista:** No es un wrapper de ChatGPT. Es un sistema RAG local, confinado a tu documentación técnica, diseñado para ser útil sin ser impredecible.

---

## Instalación

### Opción A — Descargar binario precompilado (recomendado)

Descarga el paquete para tu plataforma desde la [página de Releases](https://github.com/GerarddeTena/GeliShell/releases/latest):

| Plataforma | Archivo |
|---|---|
| Windows x64 | `geli-windows-x86_64.zip` |
| Linux x64 | `geli-linux-x86_64.tar.gz` |
| Linux ARM64 | `geli-linux-aarch64.tar.gz` |
| macOS Intel | `geli-macos-x86_64.tar.gz` |
| macOS Apple Silicon | `geli-macos-aarch64.tar.gz` |

Extrae el archivo y mueve `geli` (o `geli.exe`) a cualquier carpeta en tu `PATH`. Al ejecutar por primera vez, GeliShell descarga automáticamente sus dependencias opcionales (ver [Bootstrap automático](#bootstrap-automático)).

### Opción B — Desde el código fuente (clone local)

Para usuarios que prefieren compilar desde el repositorio:

```bash
git clone https://github.com/GerarddeTena/GeliShell.git
cd GeliShell
cargo build --release
```

Luego usa el script de instalación correspondiente:

**Windows (PowerShell):**
```powershell
.\install.ps1
.\install.ps1 -Force              # sobrescribe archivos existentes
.\install.ps1 -SkipDocs           # omite la siembra de docs.db local
.\install.ps1 -BinDir "C:\bin"   # carpeta personalizada
```

**Linux / macOS:**
```bash
./install.sh
./install.sh --force
./install.sh --skip-docs
./install.sh --bin-dir "$HOME/.local/bin"
```

> Los scripts solo copian los binarios compilados e inyectan `PATH`. No descargan `sqlite-vec` ni comprueban `sqlite3` — eso lo gestiona el propio binario al arrancar (ver [Bootstrap automático](#bootstrap-automático)).

### Rutas de instalación por defecto

| Artefacto | Windows | Linux / macOS |
|---|---|---|
| `geli` | `%USERPROFILE%\.local\bin\geli.exe` | `~/.local/bin/geli` |
| `gerisabet` | `%USERPROFILE%\.local\bin\gerisabet.exe` | `~/.local/bin/gerisabet` |
| Config | `%USERPROFILE%\.config\geliShell\` | `~/.config/geliShell/` |
| sqlite-vec | `…\models\vec0.dll` | `…/models/vec0.so` (Linux) / `vec0.dylib` (macOS) |
| RAG DB | `…\docs\docs.db` | `…/docs/docs.db` |

---

## Bootstrap automático

Al ejecutar `geli` por primera vez, el bootstrap de runtime comprueba y descarga los componentes opcionales que no estén presentes:

| Componente | Fuente | Impacto si falta |
|---|---|---|
| `sqlite-vec` (`vec0.dll` / `vec0.so` / `vec0.dylib`) | [asg017/sqlite-vec releases](https://github.com/asg017/sqlite-vec/releases/latest) | Asistente RAG no disponible |
| `docs.db` | [GeliShell releases](https://github.com/GerarddeTena/GeliShell/releases/latest) | Asistente RAG sin base de conocimiento |

**El core de GeliShell funciona al 100% sin estos componentes.** Traducción de comandos, guardrails, historial y g-jump son independientes del asistente.

Las descargas son **no bloqueantes y no fatales**: si la red no está disponible, GeliShell arranca igualmente con un aviso. La verificación SHA-256 se realiza automáticamente cuando `checksums.txt` está disponible en la release.

Para forzar que el bootstrap compruebe de nuevo, basta con eliminar el archivo correspondiente de `~/.config/geliShell/models/` o `~/.config/geliShell/docs/`.

Variables de entorno para sobrescribir las fuentes de descarga:

- `GELI_DOCS_DB_SOURCE` / `GELI_DOCS_DB_PATH` — ruta local o alternativa para `docs.db`
- `GELI_SQLITE_VEC_SOURCE` / `GELI_SQLITE_VEC_PATH` — ruta local o alternativa para `sqlite-vec`

---

## Flujo de ejecución real

Pipeline principal en `src/main.rs`:

`input -> lexer -> parser(AST) -> builtins -> guard -> translator -> executor`

Detalle por etapa:

1. **Entrada REPL** (`read_repl_input`) con historial y sugerencias tipo ghost.
2. **Lexer** (`src/parser/lexer.rs`) tokeniza con límite de 64KB.
3. **Parser** (`src/parser/parser.rs`) construye el AST.
4. **Builtins** (`BuiltinRegistry::try_execute`) se evalúan antes de traducir. Si devuelve `Handled`, el pipeline no continúa.
5. **Guard** (`default_guard()`) bloquea o exige confirmación semántica sobre el AST.
6. **TranslationPipeline** (`src/shell/translator/pipeline`) llama a `run_resolving()` que devuelve `(String, Option<ResolvedCommand>)`. Si `SelectorMode` lo requiere, muestra `ModalSelector` antes de ejecutar.
7. **Executor** (`src/shell/executor`) ejecuta async con `tokio::process::Command`.

### Steps del traductor

1. `NodeDecomposer` — único punto con `match` sobre `ASTNode`; produce `Vec<CommandFragment>`
2. `CommandResolver` — resuelve canonical→native o nativo directo (pass-through)
3. `FlagResolver` — traduce flags canónicos a nativos
4. `VariableExpander` — expande `$VAR` y `${VAR}`
5. `SubsystemMapper` — genera la string final para el subsistema activo

### Detección del ejecutable PowerShell

En Windows, el executor detecta automáticamente el ejecutable disponible en este orden: `pwsh` (PowerShell 7+) → `powershell` (Windows PowerShell 5) → ruta absoluta conocida. Esto evita el error `NotFound` en máquinas donde solo `pwsh.exe` está en `PATH`.

Todos los comandos PowerShell incluyen el preámbulo UTF-8:
```
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8;
$OutputEncoding = [System.Text.Encoding]::UTF8;
```
necesario para usuarios con caracteres no-ASCII en rutas o variables de entorno.

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
| Selector | `src/shell/selector/*` | Selector modal de alternativas de traducción |

---

## Builtins disponibles

| Comando | Comportamiento actual |
| --- | --- |
| `cd <ruta>` | Cambia directorio, actualiza `PWD`. El directorio anterior se almacena en sesión (no en `OLDPWD` del entorno) |
| `clear` | Limpia pantalla/buffer |
| `exit [code]` | Termina la shell |
| `export K=V` | Define variable de entorno de sesión |
| `unset K` | Elimina variable de entorno |
| `history` / `history --clear` | Muestra o limpia historial de sesión en memoria |
| `g` / `g <pattern>` / `g -` / `g --clear` | Navegación inteligente por frecencia |
| `source <file>` | Stub: reporta que el motor de scripting aún no está disponible |
| `gerisabet [flags]` | Detecta el binario en PATH; muestra instrucciones de instalación si no se encuentra |

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

Fuente por defecto (fallback): `src/commands/commands.toml`

Orden de carga en startup:

1. `GELI_COMMANDS_PATH` (si está definido y apunta a archivo).
2. `~/.config/geliShell/commands.toml`.
3. `commands.toml` / `commands/commands.toml` / `src/commands/commands.toml` cerca del `cwd` y del ejecutable.
4. Mapa embebido en binario (`include_str!`) como último fallback.

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
> - `docs.db` se distribuye como artefacto en la [página de Releases](https://github.com/GerarddeTena/GeliShell/releases/latest) y se descarga automáticamente en el primer arranque. No se incluye en el repositorio.
> - La construcción manual de `docs.db` (para contribuir o actualizar el conocimiento) requiere Ollama y se ejecuta con `cargo run --bin build_docs_db --features dev-tools`.
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
.\target\debug\geli.exe
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

### Rebuild de base RAG (desarrollo)

> ⚠️ Solo necesario si contribuyes al conocimiento del asistente. Requiere Ollama en ejecución y el feature `dev-tools`.

```powershell
cargo run --bin build_docs_db --features dev-tools -- --help
```

Genera/actualiza `docs.db` en `~/.config/geliShell/docs/docs.db` a partir de los Markdown en `geli-docs/`.

---

## Estructura del repositorio

```text
src/
  main.rs
  repl.rs
  lib.rs
  cli.rs
  setup.rs
  utils.rs
  gerisabet.rs
  bin/build_docs_db.rs   ← dev-only (--features dev-tools), excluido del crate publicado
  cli/
  commands/        ← TOMLs de comandos canónicos (list, copy, git, cargo…)
  handlers/
  parser/
  shell/
    assistant/
    banner.rs
    builtins/
    commands/      ← Estructuras y registro de catálogos de ecosistemas
    config/
    executor/
    guard/
    i18n/
    reporter.rs
    selector/
    translator/
    tui/
      show_me/
      ecosystem/
commands/          ← TOMLs de catálogos TUI de ecosistemas (git, npm, docker…)
  ecosystems/
docs/kb/
locales/
releases-plan.md   ← Guía para crear y gestionar GitHub Releases
install.ps1        ← Script simplificado: copia binarios + PATH (no descarga sqlite-vec)
install.sh         ← Ídem para Linux/macOS
```

---

## Desarrollo

Comandos recomendados:

```powershell
cargo check
cargo test
cargo run
```

El proyecto usa Rust 2024 (`edition = "2024"`) y crates como `tokio`, `crossterm`, `rusqlite`, `reqwest`, `serde`, `thiserror`, `flate2`, `tar`.

---

## Limitaciones y estado actual

- `source` builtin sigue en modo stub (pendiente de TriggerEngine).
- La acción `:search` está interceptada, pero la UI de búsqueda avanzada sigue en estado skeleton.
- El asistente depende de contexto RAG y síntesis local; no está en modo LLM completo de generación libre.
