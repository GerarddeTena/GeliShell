# `src/shell/builtins/g_jump/` — Navegación inteligente de directorios

El módulo `g_jump` implementa el builtin `g`: un sistema de navegación de directorios basado en **frecency** (frecuencia + recencia). Aprende los directorios que más usas y los sugiere/salta a ellos con una sola palabra clave.

---

## Uso

```bash
g                    # muestra el top 10 de directorios por score
g rust               # salta al directorio con mejor score que contiene "rust"
g -                  # vuelve al directorio anterior (equivalente a cd -)
g --clear            # limpia todo el historial de g
```

---

## Estructura

```
g_jump/
├── mod.rs          ← GJumpBuiltin — implementa el trait Builtin
├── history.rs      ← GHistory — persistencia TOML con dirty-flag
├── frequency.rs    ← Algoritmo de frecency y cálculo de scores
└── matcher.rs      ← Coincidencia fuzzy de patrones sobre rutas
```

---

## `mod.rs` — `GJumpBuiltin`

Implementa el trait `Builtin` para el comando `g`. Coordina las búsquedas, los saltos y la visualización del historial.

Recibe por inyección:
- `Arc<Mutex<GHistory>>` — historial compartido con `BuiltinRegistry`
- `Arc<Mutex<Option<PathBuf>>>` — directorio anterior compartido con `CdBuiltin` (evita escribir `OLDPWD` en el entorno del proceso)

---

## `history.rs` — `GHistory`

Gestiona la persistencia del historial en `~/.config/geliShell/g_history.toml`.

### Características clave

- **Dirty-flag**: `save()` es un no-op si no hay cambios pendientes. Evita escrituras innecesarias en cada ciclo del REPL.
- **Save on drop**: `GHistory` implementa `Drop` — persiste al salir de la sesión aunque no se llame a `save()` explícitamente.
- **Carga lazy**: si el archivo no existe, arranca con historial vacío.

### Métodos principales

```rust
history.record_visit(path)      // registra una visita (marca dirty)
history.best_match(pattern)     // devuelve la entrada con mayor score
history.top(n)                  // top N entradas ordenadas por score
history.completion_candidates(n) // rutas para el autocompletado del REPL
history.clear()                 // vacía y persiste inmediatamente
```

---

## `frequency.rs` — Algoritmo de frecency

Calcula el **score** de una entrada combinando frecuencia de visitas y tiempo transcurrido desde la última visita:

```
score = visitas × decay(ultima_visita) + case_bonus

decay:
  < 1 hora  → × 4.0
  < 1 día   → × 2.0
  < 1 semana → × 1.0
  > 1 semana → × 0.5

case_bonus (aplicado en el matcher):
  exacto case-sensitive  → +50.0
  case-insensitive       →  +0.0
  fuzzy                  → -10.0
  solo coincide path completo → -5.0
```

---

## `matcher.rs` — Coincidencia de patrones

`match_pattern(path, pattern)` determina si una ruta coincide con el patrón y devuelve el `case_bonus` asociado.

Estrategias (en orden de prioridad):
1. **Coincidencia exacta** en el basename con case correcto → bonus +50
2. **Coincidencia case-insensitive** en el basename → bonus 0
3. **Coincidencia fuzzy** (caracteres en orden) en el basename → bonus -10
4. **Coincidencia en path completo** (último recurso) → bonus -5
5. Sin coincidencia → `None`

---

## Para contribuidores

- Para modificar el **algoritmo de scoring** → edita `frequency.rs`
- Para añadir **nuevas estrategias de matching** → extiende `matcher.rs`
- Los tests usan `std::env::temp_dir()` como ruta del archivo — nunca rutas Unix hardcodeadas
