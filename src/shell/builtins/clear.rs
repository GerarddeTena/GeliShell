use super::{Builtin, BuiltinResult};
use crate::shell::reporter::Reporter;
use crossterm::{
    cursor, execute,
    terminal::{Clear, ClearType},
};
use std::io::{Write, stdout};

pub struct ClearBuiltin;

pub fn clear_console_buffer() -> std::io::Result<()> {
    let mut out = stdout();
    execute!(
        out,
        Clear(ClearType::Purge),
        Clear(ClearType::All),
        cursor::MoveTo(0, 0),
    )?;
    out.flush()?;
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
