# `src/shell/translator/` — Traducción de comandos canónicos

El traductor es el **corazón semántico** de GeliShell. Su misión: tomar el comando canónico que el usuario escribió (ej. `list`) y convertirlo en el comando nativo del subsistema activo (ej. `ls` en bash, `Get-ChildItem` en PowerShell).

---

## Estructura

```
translator/
├── mod.rs           ← Re-exporta tipos públicos
├── subsystem.rs     ← Enum Subsystem (bash, zsh, fish, powershell, cmd)
├── commands_map.rs  ← Carga y consulta del mapa de comandos (TOML)
├── resolver.rs      ← Algoritmo de sugerencias y scoring
└── pipeline/        ← Orquestador y pasos de traducción
```

---

## `subsystem.rs` — El subsistema activo

Define el enum `Subsystem` con las shells soportadas:

| Variante | Shell |
|----------|-------|
| `Subsystem::Bash` | GNU Bash |
| `Subsystem::Zsh` | Zsh |
| `Subsystem::Fish` | Fish shell |
| `Subsystem::PowerShell` | PowerShell / pwsh |
| `Subsystem::Cmd` | Windows CMD |

### Detección automática (por prioridad)
1. Variable `GELI_SUBSYSTEM` → override explícito del usuario
2. Variable `$SHELL` (solo Unix) → detecta desde la shell padre
3. Default de compilación: PowerShell en Windows, Bash en Unix/macOS

### Funciones útiles
- `subsystem.as_str()` → `"bash"`, `"powershell"`, etc.
- `subsystem.is_unix()` → `true` para bash, zsh, fish
- `subsystem.is_windows()` → `true` para powershell, cmd
- `subsystem.and_operator()` → `" && "` (o `" & "` en cmd)
- `subsystem.variable_syntax("HOME")` → `"$HOME"`, `"$env:HOME"`, `"%HOME%"`

---

## `commands_map.rs` — El mapa de comandos

**¿Qué hace?** Carga todos los archivos TOML de `src/commands/` y construye un índice en memoria para búsquedas rápidas.

Tipos principales:
- `CommandMap` — el índice principal (canónico → traducciones)
- `CommandDef` — definición de un comando con sus traducciones
- `TranslationEntry` — traducción para un subsistema concreto (`exact` + `suggestions`)
- `FlagDef` — traducción de un flag entre subsistemas

Funciones de consulta:
```rust
map.find_by_exact("list")     // busca por nombre canónico exacto
map.find_by_native("ls")      // reverse lookup: nativo → canónico
map.all_commands()            // todos los comandos para autocompletado
```

**Reverse lookup**: si el usuario escribe `ls` en PowerShell, el mapa encuentra que `ls` es la traducción bash de `list`, y devuelve `list` como canónico. El pipeline luego traduce `list` → `Get-ChildItem`.

---

## `resolver.rs` — Algoritmo de sugerencias

**¿Qué hace?** Dado un `CommandDef` y un `Subsystem`, genera una lista ordenada de sugerencias con puntuación (`ScoredSuggestion`).

Tipos de sugerencias:
- `ExactMatch` (score 100) — coincide exactamente con el `exact` del subsistema
- `NativeAlias` (score 80) — alias nativo conocido (ej. `gci` → `Get-ChildItem`)
- `NativeCommand` (score 60) — comando nativo alternativo
- `CrossPlatform` (score 40) — funciona en múltiples subsistemas

El `SuggestionResolver` implementa el trait `Resolve`, lo que permite reemplazarlo en tests.

---

## Para contribuidores

- Para añadir un **nuevo subsistema** → extiende el enum en `subsystem.rs` e implementa todos los métodos
- Para cambiar el **algoritmo de scoring** → modifica `resolver.rs`
- Los archivos TOML de comandos están en `src/commands/` — ver su README para la sintaxis
- Para entender el pipeline de traducción paso a paso → ver `pipeline/README.md`
