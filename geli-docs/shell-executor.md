# `src/shell/executor/` — Ejecución de procesos

Este módulo es responsable de **lanzar y gestionar los procesos del sistema operativo** que resultan de traducir los comandos del usuario. Es el último paso de la cadena: parser → guard → translator → **executor**.

---

## Ficheros

### `mod.rs` — `Executor`

**¿Qué hace?** El `Executor` recibe un string de comando ya traducido (ej. `"ls -la"` o `"Get-ChildItem -Force"`) y lo lanza como proceso del sistema.

Características:
- **Streaming en tiempo real**: la salida (stdout/stderr) se muestra mientras el proceso corre, sin esperar a que termine
- **Captura opcional**: si la config lo pide, también acumula la salida en memoria para uso posterior
- **Detección de TTY**: algunos programas (nvim, vim, less, man, tmux…) necesitan control total del terminal. El executor los detecta automáticamente y les pasa stdin/stdout/stderr directamente
- **Timeout**: puede matar el proceso si supera el límite configurado
- **Kill on drop**: si el proceso sigue vivo cuando se destruye el `Executor`, se mata automáticamente

**Programas que se detectan como TTY automáticamente:**
`nvim`, `vim`, `vi`, `nano`, `less`, `more`, `man`, `top`, `htop`, `tmux`, `screen`, `gerisabet`

**Añadir más programas TTY (desde config):**
```toml
[customization]
tty_commands = ["lazygit", "helix", "btop"]
```

### `config.rs` — `ExecutionConfig`

Opciones de ejecución configurables por llamada:

```rust
ExecutionConfig::minimal()              // sin captura, sin timeout
    .with_capture_output()              // captura stdout+stderr
    .with_capture_duration()            // mide el tiempo
    .with_capture_command_trace()       // guarda el string del comando
    .with_timeout(30)                   // timeout de 30 segundos
```

También disponible en modos predefinidos:
- `ExecutionConfig::minimal()` — solo ejecuta, nada más
- `ExecutionConfig::full()` — activa todas las opciones

### `result.rs` — `ExecutionResult`

Lo que devuelve el executor tras ejecutar un proceso:

```rust
pub struct ExecutionResult {
    pub exit_code: i32,          // 0 = éxito
    pub output: Option<String>,  // capturado si se pidió
    pub duration: Option<Duration>, // tiempo si se midió
    pub trace: Option<ExecTrace>,   // trazabilidad
}
```

Helper:
- `result.success()` → `true` si `exit_code == 0`
- `result.output_or_empty()` → string vacío si no se capturó

### `error.rs` — `ExecutorError`

Errores que puede devolver el executor:
- `EmptyCommand` — el string está vacío o solo espacios
- `SpawnFailed(io::Error)` — el OS no pudo lanzar el proceso
- `KilledBySignal` — el proceso fue terminado por una señal (SIGKILL, SIGTERM)
- `Timeout(secs)` — el proceso superó el tiempo límite

### `platform.rs` — Adaptación multiplataforma

**¿Qué hace?** Construye el `Command` de Tokio adaptado al subsistema activo:
- **Bash/Zsh/Fish**: `bash -c "comando"`
- **PowerShell**: `pwsh -Command "comando"` (o `powershell.exe` en Windows si `pwsh` no está)
- **Cmd**: `cmd /C "comando"`

---

## Flujo de ejecución

```
executor.run("ls -la", config, reporter)
    │
    ├─ ¿comando vacío? → Err(EmptyCommand)
    │
    ├─ ¿necesita TTY? (nvim, vim, less…)
    │   └─ Sí → spawn con stdio heredado → wait → Ok(result)
    │
    └─ No → spawn con pipes
            ├─ tokio::spawn(stream stdout → println! + capturar)
            ├─ tokio::spawn(stream stderr → eprintln! + capturar)
            ├─ wait (con o sin timeout)
            └─ Ok(ExecutionResult { exit_code, output, duration, trace })
```

---

## Para contribuidores

- Para **añadir soporte a un nuevo subsistema** (ej. nushell) → extiende `platform.rs`
- El executor es completamente **asíncrono** (tokio) — no bloquea el hilo principal
- Los tests usan `SilentReporter` y `ExecutionConfig::minimal()` para ser lo más rápidos posible
