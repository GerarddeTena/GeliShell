# `src/commands/` — Tablas de comandos canónicos

Este directorio contiene los **archivos TOML** que definen el vocabulario de GeliShell: qué comandos entiende la shell, cómo se llaman en cada subsistema nativo y qué flags admiten.

> 💡 Este directorio es donde los **usuarios avanzados y contribuidores** añaden soporte para nuevos comandos o ecosistemas.

---

## Archivos incluidos

| Archivo | Contenido |
|---------|-----------|
| `commands.toml` | Comandos generales del sistema (list, copy, move, clear, etc.) |
| `git-commands.toml` | Comandos git (status, commit, push, branch, etc.) |
| `cargo-commands.toml` | Comandos Cargo/Rust (build, test, run, check, etc.) |
| `docker-commands.toml` | Comandos Docker (build, run, ps, stop, etc.) |
| `dotnet-commands.toml` | Comandos .NET CLI (build, run, test, publish, etc.) |
| `node-commands.toml` | Comandos Node.js (node, npx, nvm, etc.) |
| `npm-commands.toml` | Comandos npm (install, run, publish, etc.) |
| `pnpm-commands.toml` | Comandos pnpm (add, install, run, etc.) |
| `typescript-commands.toml` | Comandos TypeScript (tsc, ts-node, etc.) |

---

## Estructura de un comando

Cada comando sigue este esquema TOML:

```toml
[[commands]]
name = "list"               # nombre canónico (lo que escribe el usuario)
description = "Lista el contenido del directorio actual"
category = "filesystem"

[commands.translate.bash]
exact = "ls"                # comando exacto en bash
suggestions = ["ls -la"]    # alternativas sugeridas

[commands.translate.powershell]
exact = "Get-ChildItem"
suggestions = ["gci", "dir"]

[commands.translate.zsh]
exact = "ls"
suggestions = []

[commands.translate.fish]
exact = "ls"
suggestions = []

[commands.translate.cmd]
exact = "dir"
suggestions = []
```

### Campos obligatorios
- **`name`**: identificador único del comando canónico. Es lo que el usuario escribe en GeliShell.
- **`description`**: texto breve que aparece en el menú de ayuda.
- **`category`**: agrupa comandos por área (filesystem, network, git, etc.)
- **`translate`**: traducciones para cada subsistema (`bash`, `zsh`, `fish`, `powershell`, `cmd`)

### Campos de traducción
- **`exact`**: el único comando correcto para ese subsistema. El pipeline lo usa como traducción principal.
- **`suggestions`**: lista de alternativas que el selector puede ofrecer al usuario.

---

## Flags (opcional)

Los comandos pueden incluir definiciones de flags para traducirlos entre subsistemas:

```toml
[[commands.flags]]
canonical = "--all"         # nombre canónico del flag
bash = "-a"
zsh = "-a"
powershell = "-Force"
cmd = "/A"
```

---

## ¿Cómo añadir un nuevo comando?

1. Abre el TOML apropiado (o crea uno nuevo, p.ej. `kubernetes-commands.toml`)
2. Añade una entrada `[[commands]]` con sus traducciones
3. Registra el nuevo TOML en `shell/translator/commands_map.rs` dentro de la función `load()`
4. Ejecuta los tests: `cargo test`

---

## ¿Cómo funciona el reverse lookup?

Si un usuario en PowerShell escribe `ls` (que es un comando bash), GeliShell busca en el índice inverso qué comando canónico tiene `ls` como traducción, lo encuentra (`list`) y lo traduce correctamente a `Get-ChildItem`. Esto ocurre de forma automática.

---

## Para usuarios: añadir comandos personalizados

Los usuarios también pueden añadir comandos simples sin editar los TOML, usando la configuración:

```toml
# ~/.config/geliShell/config.toml
[customization]
[[customization.custom_commands]]
name = "mi-build"
template = "cargo build --release && echo 'listo'"
```
