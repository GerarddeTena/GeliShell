# `src/` — Punto de entrada de GeliShell

Este directorio es la raíz del código fuente. Contiene el **binario principal** y la **orquestación de arranque** de la shell.

---

## Estructura de alto nivel

```
src/
├── main.rs          ← Punto de entrada del proceso
├── repl.rs          ← Bucle principal de la shell (Read-Eval-Print Loop)
├── lib.rs           ← Superficie pública de la librería
├── cli.rs           ← Parseo de argumentos de línea de comandos
├── setup.rs         ← Inicialización de componentes al arranque
├── utils.rs         ← Helpers del REPL (prompt, completados, historial)
├── gerisabet.rs     ← Punto de entrada del asistente IA como subcomando
│
├── bin/             ← Binarios auxiliares (herramientas de build/mantenimiento)
├── cli/             ← Lógica extendida de argumentos CLI
├── commands/        ← Tablas TOML de comandos canónicos
├── handlers/        ← Manejadores de eventos del REPL
├── parser/          ← Lexer + Parser → AST
└── shell/           ← Núcleo de la librería (todo lo reutilizable)
```

---

## Ficheros clave

### `main.rs`
**¿Qué hace?** Es el punto de entrada del ejecutable `geli`. Se ejecuta primero.

1. **Anti-Inception**: si detecta `GELISHELL_ACTIVE=1` en el entorno, aborta para evitar que una shell GeliShell lance otra GeliShell anidada.
2. **CLI flags**: si se pasan argumentos (`geli --help`, `geli ask "..."`) los delega a `handle_cli_args` y termina.
3. **Arranque**: inicializa `StderrReporter`, carga la configuración, detecta el idioma, carga el historial y el mapa de comandos.
4. **Montaje del REPL**: construye el `TranslationPipeline`, `Executor`, `Guard`, `BuiltinRegistry` y arranca el bucle REPL.

### `repl.rs`
**¿Qué hace?** Implementa el bucle Read→Eval→Print que convierte cada tecleo en una acción.

El bucle lee una línea de la TUI, la clasifica y actúa:
- **`help` / Ctrl+H** → abre el menú de ayuda
- **`geli-config` / Ctrl+Alt+S** → abre el menú de configuración
- **comandos especiales** (`:stop`, `:search`) → los procesa directamente
- **comandos internos geli** (`geli-helpme`, `show-me`, etc.) → `handle_geli_internal_command`
- **cualquier otra cosa** → `process_regular_command` (traduce + ejecuta)

### `lib.rs`
**¿Qué hace?** Define la superficie pública de `geli_shell` como librería (crate). Re-exporta los tipos y traits más utilizados para que otras crates (tests, herramientas externas) puedan usarlos sin conocer la jerarquía interna.

También define el macro `t!` para traducciones con o sin parámetros:
```rust
t!("repl.goodbye")
t!("executor.spawning", command = cmd, subsystem = sub)
```

### `setup.rs`
**¿Qué hace?** Agrupa las funciones de inicialización pesadas para mantener `main.rs` limpio:
- `bootstrap_runtime_layout` — crea directorios `~/.config/geliShell/`
- `load_or_init_config` — carga `config.toml` o genera uno por defecto
- `load_history_or_default` — carga `history.txt` o crea uno vacío
- `init_command_map_or_exit` — carga todos los TOML de comandos; si falla, termina el proceso con error
- `resolve_subsystem` — detecta qué shell nativa usar (bash, zsh, powershell…)

### `utils.rs`
**¿Qué hace?** Pequeñas utilidades que el REPL necesita en cada iteración:
- `render_prompt` — genera el string del prompt coloreado con ANSI256
- `build_completion_pool` — construye la lista de palabras para el autocompletado
- `append_history_or_warn` — guarda un comando en el historial persistente
- `apply_visual_settings` — aplica colores de terminal desde la config

---

## ¿Cómo fluye la ejecución?

```
geli (binario)
  └─ main.rs
       ├─ setup.rs       → carga config, historial, comandos
       └─ repl.rs        → bucle infinito
            ├─ tui/repl_input.rs   → lee teclas
            ├─ handlers/           → clasifica y despacha
            ├─ parser/             → lexer + AST
            ├─ shell/translator/   → AST → comando nativo
            ├─ shell/guard/        → comprueba seguridad
            └─ shell/executor/     → lanza el proceso
```

---

## Para contribuidores

- Para añadir un nuevo **comando builtin** → ve a `shell/builtins/`
- Para añadir soporte a un **nuevo subsistema** → ve a `shell/translator/subsystem.rs`
- Para añadir una **regla de seguridad** → ve a `shell/guard/rules/`
- Para traducir mensajes a un nuevo idioma → ve a `locales/`
