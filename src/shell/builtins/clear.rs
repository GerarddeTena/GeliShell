use super::{Builtin, BuiltinResult};
use crate::shell::reporter::Reporter;
use std::io::Write;

pub struct ClearBuiltin;

/// Limpia viewport + scrollback de forma compatible con el terminal activo.
///
/// # Windows Terminal (conpty)
/// `ClearType::Purge` de crossterm es ignorado silenciosamente por conpty.
/// La única secuencia que funciona de forma fiable es la secuencia VT raw:
///   \x1b[2J  → borra el viewport visible
///   \x1b[3J  → borra el scrollback buffer
///   \x1b[H   → mueve el cursor a (0,0)
/// El orden 2J antes de 3J es requerido por conpty.
///
/// # Unix (xterm, VTE, kitty, ...)
/// El orden invertido (3J primero) es más compatible con terminales Unix.
///
/// En ambos casos el flush es explícito — sin él conpty puede ignorar
/// la secuencia si el buffer no se vacía antes del siguiente render.
pub fn clear_console_buffer() -> std::io::Result<()> {
    #[cfg(target_os = "windows")]
    {
        print!("\x1b[2J\x1b[3J\x1b[H");
    }

    #[cfg(not(target_os = "windows"))]
    {
        print!("\x1b[3J\x1b[2J\x1b[H");
    }

    std::io::stdout().flush()?;
    Ok(())
}

impl Builtin for ClearBuiltin {
    fn name(&self) -> &'static str {
        "clear"
    }

    fn execute(&self, _args: &[String], reporter: &dyn Reporter) -> BuiltinResult {
        if let Err(error) = clear_console_buffer() {
            reporter.error(&format!("clear failed: {error}"));
        }
        BuiltinResult::Handled
    }
}
