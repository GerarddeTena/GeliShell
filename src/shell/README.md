# `src/shell/` — Núcleo de la librería GeliShell

Este es el directorio más importante del proyecto. Contiene toda la lógica de la shell empaquetada como **librería reutilizable** (`geli_shell`). Todo lo que no sea UI o arranque vive aquí.

---

## Estructura

```
shell/
├── mod.rs           ← Declara y re-exporta todos los submódulos
├── reporter.rs      ← Trait de output — cómo se muestran mensajes
├── banner.rs        ← ASCII art del arranque
│
├── assistant/       ← Asistente IA (Qwen + RAG)
├── builtins/        ← Comandos integrados (cd, g, history, exit…)
├── commands/        ← Catálogo de ecosistemas (npm, git, cargo…)
├── config/          ← Configuración persistente, bootstrap, historial
├── executor/        ← Ejecución de procesos nativos
├── guard/           ← Sistema de seguridad — bloquea comandos peligrosos
├── i18n/            ← Internacionalización (en/es y más)
├── selector/        ← Selector interactivo modal de comandos
├── translator/      ← Traducción de comandos canónicos → nativos
└── tui/             ← Interfaces de usuario en terminal
```

---

## `reporter.rs` — El contrato de output

**Regla de oro de GeliShell**: ningún módulo llama a `eprintln!` directamente. Todo el output del sistema pasa por el trait `Reporter`.

```rust
pub trait Reporter: Send + Sync {
    fn warn(&self, message: &str);
    fn error(&self, message: &str);
    fn info(&self, message: &str);
}
```

### Implementaciones disponibles

| Tipo | Uso | Descripción |
|------|-----|-------------|
| `StderrReporter` | Producción | Escribe a stderr con iconos y colores ANSI |
| `SilentReporter` | Tests unitarios | Descarta todo — evita contaminar la salida de tests |
| `BufferedReporter` | Tests de integración | Acumula mensajes en memoria para hacer asserts |

**Ejemplo en tests:**
```rust
let reporter = BufferedReporter::new();
mi_función(&reporter);
assert!(reporter.has_errors());
assert_eq!(reporter.errors()[0], "mensaje esperado");
```

**Macros de conveniencia** (evitan `format!` en hot paths):
```rust
report_info!(reporter, "ejecutando: {}", command);
report_warn!(reporter, "timeout después de {}s", secs);
report_error!(reporter, "falló con código {}", code);
```

---

## `banner.rs` — Banner de inicio

Muestra el logo ASCII de GeliShell y la versión al arrancar. Acepta cualquier `dyn Write` como destino para facilitar los tests (se puede capturar en un buffer en vez de stdout).

---

## `mod.rs`

Re-exporta todos los submódulos públicos de la shell. Es el "índice" del núcleo.

---

## Mapa de dependencias entre módulos

```
config  ──────────────────────────────────────────┐
                                                   ▼
parser ──► guard ──► translator/pipeline ──► executor
                          │
                          ▼
                       builtins
                          │
                          ▼
                    tui / selector
                          │
                          ▼
                       i18n (transversal a todo)
                       reporter (transversal a todo)
```

---

## Para contribuidores

- **Añadir un subsistema nuevo** (ej. `nushell`) → `translator/subsystem.rs`
- **Añadir un builtin nuevo** (ej. `alias`) → `builtins/`
- **Añadir una regla de seguridad** → `guard/rules/`
- **Añadir un idioma nuevo** → `i18n/` + `locales/`
- **Añadir una pantalla TUI nueva** → `tui/`
- Todo módulo nuevo debe implementar su lógica en términos del trait `Reporter` — nunca `eprintln!`
