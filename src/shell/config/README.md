# `src/shell/config/` — Configuración y estado persistente

Este módulo gestiona todo lo relacionado con la **configuración de la shell** y la **persistencia de datos** entre sesiones (historial de comandos, bootstrap del entorno).

---

## Ficheros

### `mod.rs` — `ShellConfig` (el fichero de configuración)

Define la estructura `ShellConfig` que mapea directamente al archivo `~/.config/geliShell/config.toml`.

#### Secciones de configuración

**`[behavior]`** — Comportamiento del REPL
```toml
[behavior]
selector_mode = "always"   # "always" | "auto" | "once" — cuándo mostrar el selector de sugerencias
language = "es"            # idioma de la interfaz ("en" | "es")
```

**`[subsystem]`** — Subsistema shell
```toml
[subsystem]
override_subsystem = "bash"  # fuerza un subsistema concreto (vacío = auto-detectar)
```

**`[execution]`** — Opciones de ejecución de procesos
```toml
[execution]
capture_output = false        # captura stdout/stderr en memoria
capture_duration = false      # mide el tiempo de ejecución
capture_command_trace = false # guarda traza del comando ejecutado
timeout_secs = 0              # timeout en segundos (0 = sin límite)
```

**`[visual]`** — Apariencia del prompt y terminal
```toml
[visual]
terminal_foreground_ansi256 = 253   # color del texto principal
terminal_background_ansi256 = 0     # color de fondo
prompt_path_ansi256 = 253           # color del path en el prompt
prompt_subsystem_ansi256 = 141      # color del subsistema en el prompt
prompt_name_ansi256 = 213           # color del nombre "geli" en el prompt
prompt_dim_ansi256 = 240            # color del texto fantasma (autocompletado)
font_family = "Cascadia Mono"       # fuente (solo informativo)
```

**`[customization]`** — Comandos y herramientas personalizadas
```toml
[customization]
tty_commands = ["lazygit", "helix", "btop"]  # tools que necesitan TTY interactivo

[[customization.custom_commands]]
name = "dev"
template = "cd ~/projects && nvim ."
```

**`[assistant]`** — Configuración del asistente IA
```toml
[assistant]
model_variant = "qwen-0.5b"      # variante del modelo ("qwen-0.5b" | "qwen-1.5b")
rag_top_k = 4                    # número de fragmentos RAG a recuperar
auto_unload_after_secs = 300     # descarga el modelo tras N segundos inactivo
```

#### Rutas de archivos

| Archivo | Ubicación |
|---------|-----------|
| Configuración | `~/.config/geliShell/config.toml` |
| Historial | `~/.config/geliShell/history.txt` |
| Modelos IA | `~/.config/geliShell/models/` |
| Base de datos RAG | `~/.config/geliShell/docs/docs.db` |

#### Métodos útiles
- `load_async()` — carga la configuración desde disco
- `save_async()` — persiste la configuración a disco
- `reset()` — elimina el archivo de configuración (restaura los valores por defecto al reiniciar)
- `to_executor_config()` — convierte la config en `ExecutionConfig` para el `Executor`
- `resolve_subsystem(reporter)` — detecta o fuerza el subsistema según la config

---

### `bootstrap.rs` — Arranque del entorno de runtime

**¿Qué hace?** Asegura que todos los directorios y archivos necesarios existen antes de que la shell arranque.

Acciones:
1. Crea `~/.config/geliShell/`, `models/`, `docs/` si no existen
2. **Migra** archivos de ubicaciones antiguas (legacy) a la nueva ubicación estándar
3. **Siembra** (`seed`) archivos de modelos y base de datos si están disponibles en el directorio de instalación pero no en el directorio de usuario

La semilla permite que la distribución incluya `docs.db` y el usuario no tenga que generarlo manualmente.

Variables de entorno que puede respetar el bootstrap:
- `GELI_INSTALL_ROOT` — directorio raíz de instalación (para semilla de archivos)
- `GELI_DOCS_DB_PATH` — ruta alternativa para `docs.db`
- `GELI_DOCS_DB_SOURCE` — fuente de semilla para `docs.db`
- `GELI_SQLITE_VEC_SOURCE` — fuente de semilla para la extensión vectorial

---

### `history_store.rs` — Historial de comandos persistente

**¿Qué hace?** Gestiona el historial de comandos del REPL (lo que ves con ↑/↓).

- Los comandos se guardan en `~/.config/geliShell/history.txt`, uno por línea
- `PersistentCommandHistory` carga el historial al arrancar y lo guarda tras cada comando
- `entries()` — devuelve el slice de entradas para el autocompletado del REPL

---

### `first_run.rs` — Primera ejecución

**¿Qué hace?** Detecta si es la primera vez que se ejecuta GeliShell y muestra un mensaje de bienvenida con instrucciones básicas.

---

## Para usuarios: resetear la configuración

```bash
geli-reset-config    # en el REPL
# o
rm ~/.config/geliShell/config.toml
```

Al reiniciar GeliShell, se generará una configuración nueva con los valores por defecto.
