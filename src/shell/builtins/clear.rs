use super::{Builtin, BuiltinResult};
use crate::shell::reporter::Reporter;

pub struct ClearBuiltin;

impl Builtin for ClearBuiltin {
    fn name(&self) -> &'static str { "clear" }

    fn execute(&self, _args: &[String], _reporter: &dyn Reporter) -> BuiltinResult {
        // ANSI escape para limpiar pantalla y mover cursor al inicio
        print!("\x1B[2J\x1B[1;1H");
        BuiltinResult::Handled
    }
}