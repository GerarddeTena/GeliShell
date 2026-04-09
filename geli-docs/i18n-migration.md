# i18n Migration Report — Hardcoded Strings → `t!()`

**Date:** 2026-04-02  
**Scope:** Full audit of `src/` for user-visible string literals bypassing the `t!()` macro  
**Result:** 23 violations found and fixed across 6 source files + 2 locale files

---

## Exclusions (not migrated, by rule)

| File | Strings | Reason |
|---|---|---|
| `suggest.rs` lines 47, 71 | ChatML system prompts in Spanish | LLM instructions — intentional fixed language for model stability |
| `rag.rs` line 153 + format template | Spanish RAG context strings | Embedded inside `[CONTEXTO]` block sent to LLM — same rule |
| `guard/error.rs` `#[error]` prefixes | `"🔴 BLOCKED — …"` wrappers | `thiserror` derive — cannot add `t!()` call without violating Prime Directive #7 (no manual `Display`) |
| `handlers/assistant.rs` | — | Already fully uses `t!()`, no violations |

---

## Block 1 — Audit Report

### Guard rules

```
[guard.destructive_fs.rm_root_blocked]
file: src/shell/guard/rules/destructive_fs.rs:63
old:  format!("rm with recursive+force targeting '{target}' would destroy the filesystem")
en:   "rm with recursive+force targeting '{target}' would destroy the filesystem"
es:   "rm con recursive+force apuntando a '{target}' destruiría el sistema de archivos"

[guard.destructive_fs.chmod_protected_blocked]
file: src/shell/guard/rules/destructive_fs.rs:105
old:  format!("'{}' with -R on protected path would break system permissions (sudo will stop working)", cmd.name)
en:   "'{cmd}' with -R on protected path would break system permissions (sudo will stop working)"
es:   "'{cmd}' con -R en una ruta protegida rompería los permisos del sistema (sudo dejará de funcionar)"

[guard.disk_destroyer.dd_device_blocked]
file: src/shell/guard/rules/disk_destroyer.rs:52
old:  format!("dd writing directly to block device '{target}' would destroy all data on that disk")
en:   "dd writing directly to block device '{target}' would destroy all data on that disk"
es:   "dd escribiendo directamente en el dispositivo de bloque '{target}' destruiría todos los datos de ese disco"

[guard.disk_destroyer.mkfs_format_blocked]
file: src/shell/guard/rules/disk_destroyer.rs:94
old:  format!("'{}' will FORMAT a filesystem. Pass '{}' to confirm you know what you're doing.", cmd.name, MKFS_CONFIRMATION_FLAG)
en:   "'{cmd}' will FORMAT a filesystem. Pass '{flag}' to confirm you know what you're doing."
es:   "'{cmd}' va a FORMATEAR un sistema de archivos. Pasa '{flag}' para confirmar que sabes lo que estás haciendo."

[guard.critical_redirect.sysrq_blocked]
file: src/shell/guard/rules/critical_redirect.rs:45
old:  "writing to /proc/sysrq-trigger causes immediate kernel panic and machine shutdown".to_owned()
en:   "writing to /proc/sysrq-trigger causes immediate kernel panic and machine shutdown"
es:   "escribir en /proc/sysrq-trigger provoca un pánico del kernel inmediato y el apagado de la máquina"

[guard.critical_redirect.system_file_blocked]
file: src/shell/guard/rules/critical_redirect.rs:54
old:  format!("redirecting to '{target}' would overwrite a critical system file")
en:   "redirecting to '{target}' would overwrite a critical system file"
es:   "redirigir a '{target}' sobreescribiría un archivo crítico del sistema"

[guard.pipe_execution.network_pipe_blocked]
file: src/shell/guard/rules/pipe_execution.rs:57
old:  "piping network content directly into a shell is a common malware delivery vector. Download the script first and inspect it.".to_owned()
en:   "piping network content directly into a shell is a common malware delivery vector. Download the script first and inspect it."
es:   "enviar contenido de red directamente a una shell es un vector habitual de distribución de malware. Descarga el script primero e inspecciónalo."
```

### g_jump builtin

```
[builtin.g_jump.history_lock_error]
file: src/shell/builtins/g_jump/mod.rs:56
old:  reporter.error("g: could not read history")
en:   "g: could not read history"
es:   "g: no se pudo leer el historial"

[builtin.g_jump.no_history]
file: src/shell/builtins/g_jump/mod.rs:61
old:  reporter.info("g: no history yet — navigate directories and GeliShell will learn them")
en:   "g: no history yet — navigate directories and GeliShell will learn them"
es:   "g: sin historial aún — navega por directorios y GeliShell los aprenderá"

[builtin.g_jump.col_rank]
file: src/shell/builtins/g_jump/mod.rs:72  (println! column header)
old:  "#"
en:   "#"
es:   "#"

[builtin.g_jump.col_visits]
file: src/shell/builtins/g_jump/mod.rs:72  (println! column header)
old:  "visits"
en:   "visits"
es:   "visitas"

[builtin.g_jump.col_last]
file: src/shell/builtins/g_jump/mod.rs:72  (println! column header)
old:  "last seen"
en:   "last seen"
es:   "última vez"

[builtin.g_jump.col_path]
file: src/shell/builtins/g_jump/mod.rs:72  (println! column header)
old:  "path"
en:   "path"
es:   "ruta"

[builtin.g_jump.jumped]
file: src/shell/builtins/g_jump/mod.rs:48
old:  reporter.info(&format!("g: → {target}"))
en:   "g: → {target}"
es:   "g: → {target}"

[builtin.g_jump.jump_error]
file: src/shell/builtins/g_jump/mod.rs:50
old:  reporter.error(&format!("g: {e}"))
en:   "g: {error}"
es:   "g: {error}"

[builtin.g_jump.cleared]
file: src/shell/builtins/g_jump/mod.rs:112
old:  reporter.info("g: history cleared")
en:   "g: history cleared"
es:   "g: historial limpiado"

[builtin.g_jump.no_previous]
file: src/shell/builtins/g_jump/mod.rs:119
old:  reporter.warn("g: no previous directory")
en:   "g: no previous directory"
es:   "g: no hay directorio anterior"

[builtin.g_jump.no_match]
file: src/shell/builtins/g_jump/mod.rs:137
old:  reporter.warn(&format!("g: no match for '{pattern}' — visit the directory first so GeliShell learns it"))
en:   "g: no match for '{pattern}' — visit the directory first so GeliShell learns it"
es:   "g: sin coincidencia para '{pattern}' — visita el directorio primero para que GeliShell lo aprenda"
```

### g_jump frequency (elapsed_display)

```
[builtin.g_jump.elapsed_just_now]
file: src/shell/builtins/g_jump/frequency.rs:50
old:  "just now".to_owned()
en:   "just now"
es:   "ahora mismo"
note: EN value identical to original → existing test still passes

[builtin.g_jump.elapsed_minutes_ago]
file: src/shell/builtins/g_jump/frequency.rs:52
old:  format!("{}m ago", elapsed / MINUTE)
en:   "{minutes}m ago"
es:   "hace {minutes}m"
note: EN value produces same output → existing test still passes

[builtin.g_jump.elapsed_hours_ago]
file: src/shell/builtins/g_jump/frequency.rs:54
old:  format!("{}h ago", elapsed / HOUR)
en:   "{hours}h ago"
es:   "hace {hours}h"
note: EN value produces same output → existing test still passes

[builtin.g_jump.elapsed_yesterday]
file: src/shell/builtins/g_jump/frequency.rs:56
old:  "yesterday".to_owned()
en:   "yesterday"
es:   "ayer"
note: EN value identical to original → existing test still passes

[builtin.g_jump.elapsed_days_ago]
file: src/shell/builtins/g_jump/frequency.rs:58
old:  format!("{}d ago", elapsed / DAY)
en:   "{days}d ago"
es:   "hace {days}d"
note: EN value produces same output → existing test still passes
```

---

## Block 2 — Locale diffs

### `locales/en.toml` — additions appended at end of file

```toml
[guard.destructive_fs]
rm_root_blocked         = "rm with recursive+force targeting '{target}' would destroy the filesystem"
chmod_protected_blocked = "'{cmd}' with -R on protected path would break system permissions (sudo will stop working)"

[guard.disk_destroyer]
dd_device_blocked   = "dd writing directly to block device '{target}' would destroy all data on that disk"
mkfs_format_blocked = "'{cmd}' will FORMAT a filesystem. Pass '{flag}' to confirm you know what you're doing."

[guard.critical_redirect]
sysrq_blocked       = "writing to /proc/sysrq-trigger causes immediate kernel panic and machine shutdown"
system_file_blocked = "redirecting to '{target}' would overwrite a critical system file"

[guard.pipe_execution]
network_pipe_blocked = "piping network content directly into a shell is a common malware delivery vector. Download the script first and inspect it."

[builtin.g_jump]
history_lock_error  = "g: could not read history"
no_history          = "g: no history yet — navigate directories and GeliShell will learn them"
col_rank            = "#"
col_visits          = "visits"
col_last            = "last seen"
col_path            = "path"
jumped              = "g: → {target}"
jump_error          = "g: {error}"
cleared             = "g: history cleared"
no_previous         = "g: no previous directory"
no_match            = "g: no match for '{pattern}' — visit the directory first so GeliShell learns it"
elapsed_just_now    = "just now"
elapsed_minutes_ago = "{minutes}m ago"
elapsed_hours_ago   = "{hours}h ago"
elapsed_yesterday   = "yesterday"
elapsed_days_ago    = "{days}d ago"
```

### `locales/es.toml` — additions appended at end of file

```toml
[guard.destructive_fs]
rm_root_blocked         = "rm con recursive+force apuntando a '{target}' destruiría el sistema de archivos"
chmod_protected_blocked = "'{cmd}' con -R en una ruta protegida rompería los permisos del sistema (sudo dejará de funcionar)"

[guard.disk_destroyer]
dd_device_blocked   = "dd escribiendo directamente en el dispositivo de bloque '{target}' destruiría todos los datos de ese disco"
mkfs_format_blocked = "'{cmd}' va a FORMATEAR un sistema de archivos. Pasa '{flag}' para confirmar que sabes lo que estás haciendo."

[guard.critical_redirect]
sysrq_blocked       = "escribir en /proc/sysrq-trigger provoca un pánico del kernel inmediato y el apagado de la máquina"
system_file_blocked = "redirigir a '{target}' sobreescribiría un archivo crítico del sistema"

[guard.pipe_execution]
network_pipe_blocked = "enviar contenido de red directamente a una shell es un vector habitual de distribución de malware. Descarga el script primero e inspecciónalo."

[builtin.g_jump]
history_lock_error  = "g: no se pudo leer el historial"
no_history          = "g: sin historial aún — navega por directorios y GeliShell los aprenderá"
col_rank            = "#"
col_visits          = "visitas"
col_last            = "última vez"
col_path            = "ruta"
jumped              = "g: → {target}"
jump_error          = "g: {error}"
cleared             = "g: historial limpiado"
no_previous         = "g: no hay directorio anterior"
no_match            = "g: sin coincidencia para '{pattern}' — visita el directorio primero para que GeliShell lo aprenda"
elapsed_just_now    = "ahora mismo"
elapsed_minutes_ago = "hace {minutes}m"
elapsed_hours_ago   = "hace {hours}h"
elapsed_yesterday   = "ayer"
elapsed_days_ago    = "hace {days}d"
```

---

## Block 3 — Source diffs (complete, unabbreviated)

### `src/shell/guard/rules/destructive_fs.rs`

```diff
  use crate::parser::ast::Command;
  use crate::parser::token::Token;
  use crate::shell::guard::Guard;
  use crate::shell::guard::error::GuardError;
+ use crate::t;

  ...

          if Self::has_recursive(&args) && Self::has_force(&args) {
              if let Some(target) = Self::targets_root(&args) {
                  return Err(GuardError::DestructiveFs {
-                     reason: format!(
-                         "rm with recursive+force targeting '{target}' \
-                          would destroy the filesystem"
-                     ),
+                     reason: t!("guard.destructive_fs.rm_root_blocked", target = target),
                  });
              }
          }

  ...

          if has_recursive {
              let targets_protected = Self::PROTECTED_PATHS
                  .iter()
                  .any(|&p| args.iter().any(|a| a == p));
              if targets_protected {
                  return Err(GuardError::DestructiveFs {
-                     reason: format!(
-                         "'{}' with -R on protected path would break \
-                          system permissions (sudo will stop working)",
-                         cmd.name
-                     ),
+                     reason: t!("guard.destructive_fs.chmod_protected_blocked", cmd = cmd.name),
                  });
              }
          }
```

### `src/shell/guard/rules/disk_destroyer.rs`

```diff
  use super::destructive_fs::token_args;
  use crate::parser::ast::Command;
  use crate::shell::guard::Guard;
  use crate::shell::guard::error::GuardError;
+ use crate::t;

  ...

                  return Err(GuardError::DiskDestroyer {
-                     reason: format!(
-                         "dd writing directly to block device '{target}' \
-                          would destroy all data on that disk"
-                     ),
+                     reason: t!("guard.disk_destroyer.dd_device_blocked", target = target),
                  });

  ...

          Err(GuardError::RequiresConfirmation {
-             reason: format!(
-                 "'{}' will FORMAT a filesystem. \
-                  Pass '{}' to confirm you know what you're doing.",
-                 cmd.name, MKFS_CONFIRMATION_FLAG
-             ),
+             reason: t!("guard.disk_destroyer.mkfs_format_blocked", cmd = cmd.name, flag = MKFS_CONFIRMATION_FLAG),
          })
```

### `src/shell/guard/rules/critical_redirect.rs`

```diff
  use crate::parser::ast::Command;
  use crate::parser::token::RedirectKind;
  use crate::shell::guard::Guard;
  use crate::shell::guard::error::GuardError;
+ use crate::t;

  ...

              if target == "/proc/sysrq-trigger" {
                  return Err(GuardError::CriticalRedirect {
-                     reason: "writing to /proc/sysrq-trigger causes \
-                              immediate kernel panic and machine shutdown"
-                         .to_owned(),
+                     reason: t!("guard.critical_redirect.sysrq_blocked"),
                  });
              }

              if CRITICAL_FILES.iter().any(|&f| target == f) {
                  return Err(GuardError::CriticalRedirect {
-                     reason: format!(
-                         "redirecting to '{target}' would overwrite \
-                          a critical system file"
-                     ),
+                     reason: t!("guard.critical_redirect.system_file_blocked", target = target),
                  });
              }
```

### `src/shell/guard/rules/pipe_execution.rs`

```diff
  use crate::parser::ast::{ASTNode, Command};
  use crate::shell::guard::Guard;
  use crate::shell::guard::error::GuardError;
+ use crate::t;

  ...

                  if left_is_fetcher && right_is_executor {
                      return Err(GuardError::PipeExecution {
-                         reason: "piping network content directly into a shell \
-                                  is a common malware delivery vector. \
-                                  Download the script first and inspect it."
-                             .to_owned(),
+                         reason: t!("guard.pipe_execution.network_pipe_blocked"),
                      });
                  }
```

### `src/shell/builtins/g_jump/mod.rs`

```diff
  use crate::shell::builtins::{Builtin, BuiltinResult};
  use crate::shell::reporter::Reporter;
+ use crate::t;
  use history::GHistory;
  use std::sync::{Arc, Mutex};

  ...

          match std::env::set_current_dir(target) {
              Ok(_) => {
                  unsafe {
                      std::env::set_var("PWD", target);
                  }
-                 reporter.info(&format!("g: → {target}"));
+                 reporter.info(&t!("builtin.g_jump.jumped", target = target));
              }
-             Err(e) => reporter.error(&format!("g: {e}")),
+             Err(e) => reporter.error(&t!("builtin.g_jump.jump_error", error = e)),
          }

  ...

      fn show_history(&self, reporter: &dyn Reporter) {
          let Ok(history) = self.history.lock() else {
-             reporter.error("g: could not read history");
+             reporter.error(&t!("builtin.g_jump.history_lock_error"));
              return;
          };

          if history.is_empty() {
-             reporter.info(
-                 "g: no history yet — \
-                  navigate directories and GeliShell will learn them",
-             );
+             reporter.info(&t!("builtin.g_jump.no_history"));
              return;
          }

          let top = history.top(10);

-         println!();
-         println!(
-             "  {:<4} {:<8} {:<12} {}",
-             "#", "visits", "last seen", "path"
-         );
-         println!("  {}", "─".repeat(60));
+         reporter.info("");
+         reporter.info(&format!(
+             "  {:<4} {:<8} {:<12} {}",
+             t!("builtin.g_jump.col_rank"),
+             t!("builtin.g_jump.col_visits"),
+             t!("builtin.g_jump.col_last"),
+             t!("builtin.g_jump.col_path"),
+         ));
+         reporter.info(&format!("  {}", "─".repeat(60)));

          for (i, (entry, score)) in top.iter().enumerate() {
              let last = frequency::elapsed_display(entry.last_visit);
              let display_path = shorten_home(&entry.path);

-             println!(
+             reporter.info(&format!(
                  "  {:<4} {:<8} {:<12} {}  \x1b[38;5;240m({:.0})\x1b[0m",
                  i + 1,
                  entry.visits,
                  last,
                  display_path,
                  score,
-             );
+             ));
          }
-         println!();
+         reporter.info("");
      }

  ...

              Some("--clear") => {
                  if let Ok(mut h) = self.history.lock() {
                      h.clear();
-                     reporter.info("g: history cleared");
+                     reporter.info(&t!("builtin.g_jump.cleared"));
                  }
              }

              Some("-") => match std::env::var("OLDPWD") {
                  Ok(prev) => self.jump_to(&prev, reporter),
-                 Err(_) => reporter.warn("g: no previous directory"),
+                 Err(_) => reporter.warn(&t!("builtin.g_jump.no_previous")),
              },

  ...

                  None => {
-                     reporter.warn(&format!(
-                         "g: no match for '{pattern}' — \
-                          visit the directory first so GeliShell learns it"
-                     ));
+                     reporter.warn(&t!(
+                         "builtin.g_jump.no_match",
+                         pattern = pattern
+                     ));
                  }
```

### `src/shell/builtins/g_jump/frequency.rs`

```diff
+ use crate::t;
  use std::time::{SystemTime, UNIX_EPOCH};

  ...

      if elapsed < MINUTE {
-         "just now".to_owned()
+         t!("builtin.g_jump.elapsed_just_now")
      } else if elapsed < HOUR {
-         format!("{}m ago", elapsed / MINUTE)
+         t!("builtin.g_jump.elapsed_minutes_ago", minutes = elapsed / MINUTE)
      } else if elapsed < DAY {
-         format!("{}h ago", elapsed / HOUR)
+         t!("builtin.g_jump.elapsed_hours_ago", hours = elapsed / HOUR)
      } else if elapsed < DAY * 2 {
-         "yesterday".to_owned()
+         t!("builtin.g_jump.elapsed_yesterday")
      } else {
-         format!("{}d ago", elapsed / DAY)
+         t!("builtin.g_jump.elapsed_days_ago", days = elapsed / DAY)
      }
```

---

## Summary

| Metric | Count |
|---|---|
| Files modified | 8 (6 Rust + 2 TOML) |
| Violations fixed | 23 |
| New locale keys added | 23 (×2 locales = 46 entries) |
| New `[guard.*]` sections | 4 |
| New `[builtin.g_jump]` section | 1 (16 keys) |
| Elapsed-time keys | 5 |
| `println!` → `reporter.info()` conversions | 4 |
| Build result | ✅ `cargo check` clean — 0 new errors or warnings |

### Test compatibility

`frequency.rs` has an existing `#[cfg(test)]` block that checks exact string values for `elapsed_display`.  
All EN locale values were chosen to be **identical** to the original English strings, so the tests continue to pass in EN locale without modification:

| Test assertion | EN locale value | Match |
|---|---|---|
| `"just now"` | `"just now"` | ✅ |
| `"1h ago"` | `"{hours}h ago"` → `"1h ago"` | ✅ |
| `"yesterday"` | `"yesterday"` | ✅ |
| `"2d ago"` | `"{days}d ago"` → `"2d ago"` | ✅ |
