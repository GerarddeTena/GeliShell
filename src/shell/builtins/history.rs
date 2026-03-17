// Stub — history se maneja directamente en BuiltinRegistry
// porque necesita acceso al Vec<String> interno
use super::{Builtin, BuiltinResult};
use crate::shell::reporter::Reporter;

pub struct HistoryBuiltin;

impl Builtin for HistoryBuiltin {
    fn name(&self) -> &'static str {
        "history"
    }
    fn execute(&self, _args: &[String], _reporter: &dyn Reporter) -> BuiltinResult {
        BuiltinResult::Handled
    }
}
