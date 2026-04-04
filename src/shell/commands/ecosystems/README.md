# `src/shell/commands/ecosystems/` — Estructuras y registro de ecosistemas

Este directorio contiene el **corazón del sistema de catálogos** de GeliShell: las estructuras de datos que representan un ecosistema y el mecanismo que los carga desde archivos TOML.

---

## Ficheros

### `mod.rs` — Tipos del dominio

Define los tipos que modelan un catálogo de comandos de ecosistema:

| Tipo | Descripción |
|------|-------------|
| `EcosystemCatalog` | Catálogo completo de un ecosistema (meta + operaciones) |
| `EcosystemMeta` | Metadatos: nombre, descripción, niveles de dificultad |
| `EcosystemOperation` | Una operación nombrada con su nivel y sus comandos |
| `EcosystemCommand` | Un comando concreto con su subsistema destino |

El campo `subsystem` de `EcosystemCommand` puede ser:
- `"bash"`, `"zsh"`, `"fish"` — solo para shells Unix
- `"powershell"`, `"cmd"` — solo para shells Windows
- `"all"` — funciona en cualquier subsistema

### `registry.rs` — Cargador de catálogos

**¿Qué hace?** Lee los archivos TOML de `commands/ecosystems/` y los deserializa en `EcosystemCatalog`.

Expone métodos para obtener catálogos por nombre:
```rust
// Carga todos los catálogos disponibles
let registry = EcosystemRegistry::load()?;

// Obtiene el catálogo de git
let catalog = registry.get("git");

// Lista los nombres disponibles
let names = EcosystemRegistry::available();
```

Catálogos cargados:
- `git.toml` — comandos Git
- `npm.toml` — comandos npm
- `pnpm.toml` — comandos pnpm
- `cargo-lang.toml` — comandos Cargo/Rust
- `docker.toml` — comandos Docker
- `dotnet.toml` — comandos .NET
- `node.toml` — comandos Node.js
- `python.toml` — comandos Python/pip
- `typescript.toml` — comandos TypeScript

Si el nombre no existe o el TOML está mal formado, devuelve un error descriptivo.

---

## Relación con la TUI

La TUI de ecosistemas (`shell/tui/ecosystem/`) consume directamente estos tipos para renderizar su interfaz de dos paneles (Operaciones → Comandos → Detalle).

---

## Para contribuidores

Ver [`commands/README.md`](../README.md) para instrucciones de cómo añadir un nuevo ecosistema.
