# `src/shell/commands/` — Catálogo de ecosistemas

Este módulo define las **estructuras de datos** para los catálogos de ecosistemas (npm, git, cargo, docker…) y el registro que los carga desde los archivos TOML de `commands/ecosystems/`.

> 💡 Los ecosistemas son colecciones de comandos agrupados por herramienta que se visualizan en la TUI `show-me`. No son lo mismo que los comandos canónicos de `src/commands/`.

---

## Estructura

```
commands/
├── mod.rs           ← Re-exporta ecosystems
└── ecosystems/
    ├── mod.rs       ← Estructuras de datos del catálogo
    └── registry.rs  ← Carga y registro de catálogos desde TOML
```

---

## `ecosystems/mod.rs` — Estructuras del catálogo

Define las estructuras que representan un catálogo de ecosistema:

```rust
pub struct EcosystemCatalog {
    pub meta: EcosystemMeta,         // nombre, descripción, niveles
    pub ops: Vec<EcosystemOperation>, // lista de operaciones
}

pub struct EcosystemOperation {
    pub operation: String,           // nombre de la operación (ej. "Instalar dependencias")
    pub level: String,               // nivel de dificultad (basic, intermediate, advanced)
    pub commands: Vec<EcosystemCommand>,
}

pub struct EcosystemCommand {
    pub subsystem: String,           // "bash", "powershell", "all"
    pub command: String,             // el comando literal a ejecutar
}
```

---

## `ecosystems/registry.rs` — Registro de ecosistemas

**¿Qué hace?** Carga todos los catálogos de ecosistemas disponibles y los pone a disposición de la TUI.

Los catálogos se cargan desde `commands/ecosystems/*.toml` (directorio `commands/ecosystems/` en la raíz del proyecto, distinto de este directorio `src/shell/commands/`).

Catálogos incluidos por defecto:
- `git.toml` — comandos Git
- `npm.toml` — comandos npm
- `pnpm.toml` — comandos pnpm
- `cargo-lang.toml` — comandos Cargo/Rust
- `docker.toml` — comandos Docker
- `dotnet.toml` — comandos .NET
- `node.toml` — comandos Node.js
- `python.toml` — comandos Python/pip
- `typescript.toml` — comandos TypeScript

---

## ¿Cómo se usa en la shell?

```bash
show-me git       # abre la TUI con todos los comandos Git organizados
show-me npm       # abre la TUI con comandos npm
show-me list      # lista todos los ecosistemas disponibles
```

---

## Para contribuidores: añadir un nuevo ecosistema

1. Crea un archivo TOML en `commands/ecosystems/`, p.ej. `kubernetes.toml`:
```toml
[meta]
name = "kubernetes"
description = "Comandos kubectl para gestión de clústeres"
levels = ["basic", "intermediate", "advanced"]

[[ops]]
operation = "Ver pods"
level = "basic"
commands = [
  { subsystem = "all", command = "kubectl get pods" },
  { subsystem = "all", command = "kubectl get pods -A" },
]

[[ops]]
operation = "Ver logs"
level = "basic"
commands = [
  { subsystem = "all", command = "kubectl logs <pod-name>" },
]
```

2. Registra el nuevo TOML en `ecosystems/registry.rs`
3. La TUI lo cargará automáticamente con su tema de color por defecto (o puedes añadir un tema en `tui/ecosystem/mod.rs`)
