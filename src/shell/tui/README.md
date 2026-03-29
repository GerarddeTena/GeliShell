# `src/shell/tui/` — Interfaces de usuario en terminal

Este directorio contiene todos los **componentes de interfaz visual** de GeliShell: el REPL interactivo, los menús, el explorador de ecosistemas y el catálogo de documentación.

---

## Estructura

```
tui/
├── mod.rs              ← Declara submódulos
├── repl_input.rs       ← Entrada del REPL (teclas, historial, autocompletado)
├── help_menu.rs        ← Menú de ayuda interactivo
├── config_menu.rs      ← Menú de configuración interactivo
├── assistant_menu.rs   ← Interfaz del asistente IA
├── ecosystem/          ← TUI de exploración de ecosistemas (show-me)
└── show_me/            ← Catálogo de documentación y placeholders
```

---

## `repl_input.rs` — La línea de entrada del REPL

**¿Qué hace?** Es el componente que muestra el prompt y gestiona toda la interacción de teclado mientras el usuario escribe un comando.

### Funcionalidades
- **Edición de línea**: mover cursor (←/→), borrar (Backspace/Delete), Home/End
- **Historial**: ↑/↓ navega por comandos anteriores, conservando el borrador actual
- **Autocompletado ghost**: mientras escribes, aparece una sugerencia en gris que puedes aceptar con → o Tab
- **Completado inteligente**: prioridad historial > paths de `g` > pool de comandos
- **Pegado**: soporta pegado de texto sin romper el cursor
- **Flush de eventos stale**: previene que un ENTER buffereado durante la ejecución de un proceso dispare un comando vacío en el siguiente ciclo (bug del doble ENTER)

### Atajos de teclado

| Atajo | Acción |
|-------|--------|
| `Enter` | Ejecutar el comando |
| `Ctrl+D` | Salir de la shell |
| `Ctrl+H` o `Ctrl+?` | Abrir menú de ayuda |
| `Ctrl+L` | Limpiar pantalla |
| `Ctrl+S` | Abrir búsqueda |
| `Ctrl+Alt+S` | Abrir menú de configuración |
| `Ctrl+Alt+G` | Abrir asistente IA |
| `→` o `Tab` | Aceptar sugerencia ghost |
| `↑`/`↓` | Navegar historial |

### `ReplInputAction`
Lo que devuelve `read_repl_input()`:
```rust
pub enum ReplInputAction {
    Command(String),   // texto a ejecutar
    Exit,              // Ctrl+D
    OpenHelp,          // Ctrl+H
    OpenConfig,        // Ctrl+Alt+S
    OpenAssistant,     // Ctrl+Alt+G
    Clear,             // Ctrl+L
    Search,            // Ctrl+S
}
```

---

## `help_menu.rs` — Menú de ayuda

**¿Qué hace?** Muestra una interfaz TUI con todos los comandos canónicos disponibles, sus descripciones y sus traducciones para el subsistema activo. Navegable con cursor.

Se abre con: `Ctrl+H`, escribir `help`, o el atajo en el prompt.

---

## `config_menu.rs` — Menú de configuración

**¿Qué hace?** Interfaz TUI para **editar la configuración de la shell** sin tocar el archivo TOML manualmente. Muestra todas las opciones con sus valores actuales y permite modificarlas interactivamente.

Se abre con: `Ctrl+Alt+S`, escribir `geli-config`, o `geli-reset-config` para resetear.

Después de guardar, la configuración se recarga en caliente (sin reiniciar la shell).

---

## `assistant_menu.rs` — Interfaz del asistente IA

**¿Qué hace?** Muestra la respuesta del asistente IA con formato enriquecido: código resaltado, sugerencias numeradas, explicaciones. También muestra el progreso durante la carga del modelo.

---

## Para contribuidores

- Todos los componentes TUI usan la librería **crossterm** para manipulación del terminal multiplataforma
- El patrón es: `enable_raw_mode()` → bucle de eventos → `disable_raw_mode()`
- Siempre usa `RawModeGuard` o un equivalente RAII para garantizar que raw mode se desactiva aunque haya un panic
- Para añadir una **nueva pantalla TUI**, crea un archivo nuevo aquí y regístrala en `handlers/menu.rs`
