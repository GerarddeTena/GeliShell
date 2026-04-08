# `src/handlers/` — Manejadores de eventos del REPL

Este directorio contiene las **funciones que procesan cada tipo de entrada** que el usuario puede escribir en el REPL. Actúan como un router de segundo nivel: una vez que `repl.rs` clasifica la entrada, delega en uno de estos handlers.

---

## Ficheros

### `mod.rs`
Declara y re-exporta los submódulos del directorio.

### `command.rs`
**¿Qué hace?** Maneja la **ejecución de comandos regulares** — todo lo que no es un builtin ni un comando interno de geli.

Flujo interno:
1. Parsea el texto con el lexer y el parser → obtiene un `ASTNode`
2. Comprueba el AST contra el **Guard** de seguridad → si hay riesgo, aborta con mensaje
3. Intenta ejecutarlo como **builtin** (cd, history, g, etc.) → si se maneja, continúa
4. Ejecuta el pipeline de **traducción** → obtiene el comando nativo
5. Si hay sugerencias disponibles y el modo selector está activo, abre el **selector modal**
6. Lanza el **executor** para ejecutar el proceso
7. Llama a `drain_crossterm_events` para limpiar eventos de teclado stale tras la ejecución

### `geli_internal.rs`
**¿Qué hace?** Maneja los **comandos propios de GeliShell** — aquellos que empiezan por `geli-` o tienen nombres especiales como `show-me`.

| Comando | Acción |
|---------|--------|
| `geli-helpme <pregunta>` | Consulta al asistente IA con una pregunta |
| `show-me <ecosistema>` | Abre la TUI del catálogo de comandos (npm, git, cargo…) |
| `show-me list` | Lista los ecosistemas disponibles |
| `geli-reset-config` | Borra `config.toml` y restaura los valores por defecto |

### `menu.rs`
**¿Qué hace?** Gestiona la apertura de los **menús TUI** de GeliShell:

- `handle_help_menu` — abre el menú interactivo de ayuda (Ctrl+H o escribir `help`)
- `handle_config_menu` — abre el menú interactivo de configuración (Ctrl+Alt+S o `geli-config`)
- `handle_special_command` — procesa comandos especiales como `:stop` y `:search`
- `is_help_trigger` / `is_config_trigger` — detectan si la entrada del usuario activa un menú

### `assistant.rs`
**¿Qué hace?** Encapsula la lógica de **interacción con el asistente IA** dentro del REPL:
- Inicializa el `AssistantRuntime` si no existe
- Muestra una barra de progreso durante la carga del modelo
- Llama a `run_how_to` o `run_parameter` y muestra el resultado formateado
- Gestiona la descarga del modelo cuando lleva tiempo inactivo

---

## Diagrama de flujo del handler principal

```
Usuario escribe: "list -a"
        │
        ▼
command.rs::process_regular_command()
        │
        ├─ parser: "list -a" → ASTNode::Command { name: "list", args: ["-a"] }
        │
        ├─ guard.check(ast) → OK (no es destructivo)
        │
        ├─ builtins.try_execute(ast) → NotABuiltin
        │
        ├─ pipeline.run(ast) → "ls -a"  (en bash)
        │               └─ "Get-ChildItem -Force" (en PowerShell)
        │
        └─ executor.run("ls -a") → output en pantalla
```

---

## Para contribuidores

- Para **añadir un nuevo comando `geli-*`** → extiende `parse_geli_internal_command` y `handle_geli_internal_command` en `geli_internal.rs`
- Para **añadir un nuevo menú TUI** → crea la lógica en `shell/tui/` y regístrala en `menu.rs`
- Todos los handlers reciben `&dyn Reporter` — nunca usen `eprintln!` directamente
