# `src/shell/tui/ecosystem/` — TUI del explorador de ecosistemas

Este directorio implementa la **interfaz visual de pantalla completa** para explorar los catálogos de comandos de ecosistemas (npm, git, cargo, docker, etc.).

> 💡 Se activa con el comando `show-me <ecosistema>`, por ejemplo: `show-me git`

---

## Ficheros

### `mod.rs` — `EcosystemTui`

**¿Qué hace?** Renderiza una interfaz de **tres paneles** que permite explorar y ejecutar comandos de un ecosistema concreto.

```
┌─────────────────────────────────────────────────────────────┐
│ 🌲 GeliShell Ecosystem — GIT  :: 24 commands loaded         │
├────────────────────┬───────────────────┬────────────────────┤
│ Operations         │ Commands          │ Detail             │
│─────────────────── │───────────────────│────────────────────│
│ > Ver estado       │ > git status      │ Operation: Ver est │
│   Crear rama       │   git status -s   │ Level: basic       │
│   Cambiar rama     │                   │ Subsystem: bash    │
│   Hacer commit     │                   │ Description: ...   │
│   ...              │                   │ Visible commands: 2│
├────────────────────┴───────────────────┴────────────────────┤
│ TAB Panel  RET Exec  s Subsystem: bash  / Filter  Q Quit    │
└─────────────────────────────────────────────────────────────┘
```

### Controles de la TUI

| Tecla | Acción |
|-------|--------|
| `Tab` | Cambia el foco entre el panel Operaciones y el panel Comandos |
| `↑`/`↓` | Navega dentro del panel activo |
| `Enter` | Selecciona el comando y pide confirmación de ejecución |
| `s` | Alterna entre filtrar por subsistema activo o mostrar todos |
| `/` | Activa el modo filtro — escribe para buscar operaciones |
| `Esc`/`q` | Cierra la TUI y vuelve al REPL |

### Confirmación de ejecución
Antes de ejecutar, la TUI sale del modo pantalla completa y muestra:
```
git status -s

Execute? [y/N]:
```
Si el usuario confirma, el comando se devuelve al REPL para ejecutarlo.

### Placeholders interactivos
Si el comando contiene `<placeholders>` (ej. `git checkout <branch>`), la TUI pausa para que el usuario introduzca el valor antes de ejecutar.

### Temas por ecosistema

| Ecosistema | Color | Icono |
|------------|-------|-------|
| npm | Rojo | 📦 |
| git | Verde | 🌲 |
| cargo | Amarillo oscuro | 🦀 |
| docker | Azul | 🐳 |
| dotnet | Magenta | 🟣 |
| python | Amarillo | 🐍 |
| (otros) | Cyan | ⚡ |

### `error.rs` — `EcosystemTuiError`
Errores específicos de la TUI del ecosistema (problemas de terminal, fallos de renderizado).

---

## Para contribuidores

- Para añadir un **nuevo ecosistema con tema** → añade un caso en `get_theme()` en `mod.rs`
- La TUI entra en `terminal::EnterAlternateScreen` — siempre limpia con `LeaveAlternateScreen` al salir, incluso en errores
- Los comandos con placeholders se resuelven via `show_me::resolve_placeholders_for_tui()`
