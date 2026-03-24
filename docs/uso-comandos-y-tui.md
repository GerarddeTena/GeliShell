# GeliShell - Guia de comandos y TUI

Este documento describe **como usar los comandos CLI** y **como acceder a las TUI que hoy estan implementadas** en el estado actual del proyecto.

## 1) Binarios y responsabilidades

GeliShell esta dividido en dos binarios:

- `geli`: shell principal + TUI de configuracion, ayuda y ecosistemas.
- `gerisabet`: asistente + TUI de catalogo RAG (`--show-me`).

## 2) Comandos CLI de `geli`

### Ayuda

```powershell
geli --help
```

### Menu de configuracion (TUI)

```powershell
geli --config-me
```

### Catalogo por ecosistema (TUI)

```powershell
geli --show --commands cargo
geli --show --commands docker
geli --show --commands dotnet
geli --show --commands git
geli --show --commands npm
geli --show --commands python
```

Si usas un ecosistema no valido, `geli` muestra la lista permitida.

## 3) Comandos CLI de `gerisabet`

### Ayuda

```powershell
gerisabet --help
```

### Asistencia guiada (`--how-to`)

```powershell
gerisabet --how-to "como comprimir una carpeta"
```

### Catalogo RAG interactivo (`--show-me`)

```powershell
gerisabet --show-me
```

## 4) TUI existentes y como acceder

## TUI expuestas al usuario final

- `Config Menu`
  - Acceso: `geli --config-me`
  - Objetivo: editar colores/fuente y abrir flujo TOML de comandos.

- `Help Menu`
  - Acceso: dentro del REPL de `geli` con `Ctrl+H` o `Ctrl+?`.
  - Objetivo: ver atajos y acciones rapidas (`clear`, `stop`, `search`, `exit`).

- `Ecosystem TUI`
  - Acceso: `geli --show --commands <ecosystem>`
  - Objetivo: explorar operaciones por ecosistema y ejecutar comandos sugeridos.

- `Show-Me TUI (assistant/RAG)`
  - Acceso: `gerisabet --show-me`
  - Objetivo: navegar catalogo generado desde `docs.db`, resolver placeholders y confirmar ejecucion.

## TUI implementadas pero no expuestas por comando directo

- `Assistant Menu`
  - Esta implementada en `src/shell/tui/assistant_menu.rs`.
  - Actualmente **no tiene flag CLI publico** en `gerisabet` (el flujo activo usa `--how-to` y `--show-me`).

## 5) Navegacion rapida por TUI

## `Ecosystem TUI` (`geli --show --commands ...`)

- `Tab`: alterna panel (`Operations` / `Commands`).
- `Up` / `Down`: navegar lista activa.
- `Enter` (en panel comandos): resolver placeholders y confirmar ejecucion.
- `/`: entrar en modo filtro por texto.
- `s`: alternar filtro por subsistema (solo activo / todos).
- `q` o `Esc`: salir.

## `Show-Me TUI` (`gerisabet --show-me`)

- Vista A: categorias.
- Vista B: tabla de comandos filtrada por subsistema (con fallback visible si no hay match).
- `Enter`: seleccionar comando, resolver placeholders `<marker>`, confirmar con `Execute? [y/N]`.
- `Esc`, `Backspace` o `q`: volver/salir segun estado.

## `Config Menu` (`geli --config-me`)

- `Up` / `Down`: mover fila.
- `Left` / `Right`: mover columna.
- `Enter`: ejecutar accion seleccionada.
- `Esc` o `q`: cerrar.

## `Help Menu` (desde REPL)

- `Up` / `Down`: mover fila.
- `Left` / `Right`: mover columna.
- `Enter`: ejecutar accion de la fila.
- `Esc` o `q`: cerrar.

## 6) Atajos del REPL de `geli`

Atajos activos en captura de input:

- `Ctrl+D`: salir.
- `Ctrl+H` o `Ctrl+?`: abrir Help Menu.
- `Ctrl+L`: clear.
- `Ctrl+Alt+S`: abrir Config Menu.
- `Ctrl+Alt+G`: muestra aviso para usar `gerisabet --help`.
- `Ctrl+S`: accion search.
- `Tab` y `Right`: aceptar sugerencia ghost.
- `Up` / `Down`: historial.

Comandos especiales interceptados:

- `:stop` / `:stop*`
- `:search` / `:search*`

## 7) Flujo recomendado de uso

Para shell diaria:

```powershell
geli
```

Para configurar UI:

```powershell
geli --config-me
```

Para explorar recetas por ecosistema:

```powershell
geli --show --commands git
```

Para asistencia RAG:

```powershell
gerisabet --how-to "listar archivos grandes"
gerisabet --show-me
```

