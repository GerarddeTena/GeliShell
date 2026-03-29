# `src/shell/tui/show_me/` — Catálogo de documentación y placeholders

Este directorio implementa la lógica del comando `show-me`: el **catálogo de documentación** en base de datos SQLite y la resolución interactiva de **placeholders** en comandos.

---

## Ficheros

### `mod.rs`
Punto de entrada del módulo. Expone las funciones públicas utilizadas por la TUI del ecosistema y los handlers del REPL.

Funciones principales:
- `resolve_placeholders_for_tui(command, out)` — detecta los placeholders `<nombre>` en un comando y pregunta al usuario por cada uno interactivamente
- `subsystem_matches_for_tui(cmd_subsystem, active)` — comprueba si un comando de ecosistema es compatible con el subsistema activo del usuario

### `catalog.rs` — Consulta del catálogo

**¿Qué hace?** Realiza búsquedas en la base de datos de documentación (`docs.db`) para el comando `show-me`.

Permite buscar por:
- Nombre de ecosistema
- Categoría de operación
- Texto libre

### `db.rs` — Conexión a la base de datos

**¿Qué hace?** Gestiona la conexión SQLite a `docs.db`. Usa la extensión `sqlite-vec` para búsquedas vectoriales cuando el asistente IA lo necesita.

Rutas buscadas (en orden de prioridad):
1. Variable de entorno `GELI_DOCS_DB_PATH`
2. `~/.config/geliShell/docs/docs.db`

### `placeholder.rs` — Resolución interactiva de placeholders

**¿Qué hace?** Detecta tokens del tipo `<nombre>` en un string de comando y solicita al usuario que los rellene uno a uno.

**Ejemplo:**
```
Comando: git checkout <branch-name>

GeliShell solicita:
  branch-name: feature/mi-nueva-funcionalidad

Resultado: git checkout feature/mi-nueva-funcionalidad
```

Soporta múltiples placeholders en el mismo comando:
```
Comando: git push <remote> <branch>
  remote: origin
  branch: main
→ git push origin main
```

### `error.rs` — `ShowMeError`
Errores del módulo: base de datos no encontrada, error de consulta, fallo de terminal durante la resolución de placeholders.

---

## Relación con otros módulos

```
handlers/geli_internal.rs
    └─ "show-me git"
         └─ commands/ecosystems/registry.rs → carga EcosystemCatalog
              └─ tui/ecosystem/mod.rs → renderiza TUI
                   └─ tui/show_me/placeholder.rs → resuelve placeholders
                        └─ tui/show_me/db.rs → docs.db (para asistente)
```

---

## Para contribuidores

- Los placeholders usan la convención `<nombre-descriptivo>` (entre ángulos, con guiones)
- Para añadir soporte a placeholders opcionales (con valor por defecto) → extiende `placeholder.rs`
- La base de datos `docs.db` se genera con `src/bin/build_docs_db.rs`
