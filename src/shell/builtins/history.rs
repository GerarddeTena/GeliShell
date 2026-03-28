// history is handled as a special case in BuiltinRegistry::try_execute
// because it requires access to the internal command history Vec.
// HistoryBuiltin is not registered in the builtins vec — it exists as
// a placeholder that signals future refactoring to the Arc<Mutex<Vec>> pattern
// (see GJumpBuiltin for the established approach).
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
