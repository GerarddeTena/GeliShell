# `src/shell/guard/rules/` — Reglas de seguridad individuales

Este directorio contiene cada una de las **reglas de seguridad** que componen el guard de GeliShell. Cada archivo implementa una categoría específica de comandos peligrosos.

---

## Ficheros y reglas

### `mod.rs`
Re-exporta todas las reglas para que `guard/mod.rs` pueda importarlas limpiamente.

### `destructive_fs.rs` — Operaciones destructivas en el sistema de archivos

**`RmGuard`** — Bloquea eliminaciones masivas o en rutas críticas:
- `rm -rf /` — eliminar el sistema de archivos raíz
- `rm -rf ~` — eliminar el home del usuario
- `rm -rf *` desde directorios críticos
- Variantes con flags en diferente orden (`rm -r -f`, `rm --force --recursive`)

**`ChmodChownGuard`** — Bloquea cambios de permisos/propietario en rutas del sistema:
- `chmod 777 /etc`
- `chown root /`
- Permisos que eliminan la protección de directorios críticos

### `disk_destroyer.rs` — Operaciones de bajo nivel sobre discos

**`DdGuard`** — Bloquea comandos `dd` que sobreescriben dispositivos de bloque:
- `dd if=/dev/zero of=/dev/sda`
- `dd if=archivo.iso of=/dev/sdb`

**`MkfsGuard`** — Bloquea formateos de dispositivos:
- `mkfs.ext4 /dev/sda1`
- `mkfs.ntfs /dev/sdb`

### `critical_redirect.rs` — Redirecciones a rutas críticas

Bloquea redirigir salida a archivos que podrían corromper el sistema:
- `echo "" > /dev/sda` — sobreescribe directamente el disco
- `command > /etc/passwd` — sobrescribe el archivo de usuarios
- `command > /etc/shadow` — sobrescribe el archivo de contraseñas
- Redirecciones a `/proc/`, `/sys/` y otros pseudo-filesystems

### `fork_bomb.rs` — Detección de fork bombs

Detecta patrones de fork bomb que colapsan el sistema creando procesos en bucle:
- `:() { :|: & }; :` — la fork bomb clásica de bash
- Variantes con nombres de función diferentes
- Invocaciones recursivas de función con background

### `pipe_execution.rs` — Ejecución de scripts remotos

Bloquea el patrón de descargar y ejecutar código sin verificación:
- `curl URL | bash`
- `curl URL | sh`
- `wget -O- URL | bash`
- `fetch URL | sh`

> **¿Por qué es peligroso?** El script puede cambiar entre el momento en que el servidor lo sirve y cuando se ejecuta. Es mejor: descargar → revisar → ejecutar.

---

## Diseño de las reglas

Cada regla:
1. Solo implementa `check_command(&self, cmd: &Command) -> Result<(), GuardError>`
2. Analiza `cmd.name` (ya normalizado por `NormalizedCompositeGuard`) y `cmd.args`
3. Devuelve `Ok(())` si es seguro, `Err(GuardError::new("mensaje"))` si no lo es
4. **Nunca** ejecuta código externo ni tiene efectos secundarios
5. **Nunca** modifica el comando — solo lo evalúa

---

## Para contribuidores

Plantilla para una nueva regla:
```rust
// src/shell/guard/rules/mi_regla.rs

use crate::parser::ast::Command;
use crate::shell::guard::{Guard, GuardError};

pub struct MiReglaGuard;

impl MiReglaGuard {
    pub fn new() -> Self { Self }
}

impl Guard for MiReglaGuard {
    fn check_command(&self, cmd: &Command) -> Result<(), GuardError> {
        if cmd.name == "comando_peligroso" {
            return Err(GuardError::new(
                "razón por la que es peligroso y qué hacer en su lugar"
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::token::Token;

    fn make_cmd(name: &str, args: &[&str]) -> Command {
        Command {
            name: name.to_owned(),
            args: args.iter().map(|a| Token::Word(a.to_string())).collect(),
            redirections: vec![],
        }
    }

    #[test]
    fn blocks_dangerous_command() {
        let guard = MiReglaGuard::new();
        assert!(guard.check_command(&make_cmd("comando_peligroso", &[])).is_err());
    }

    #[test]
    fn allows_safe_command() {
        let guard = MiReglaGuard::new();
        assert!(guard.check_command(&make_cmd("echo", &["hello"])).is_ok());
    }
}
```
