use super::{Builtin, BuiltinResult};
use crate::shell::reporter::Reporter;

pub struct ExitBuiltin;

impl Builtin for ExitBuiltin {
    fn name(&self) -> &'static str {
        "exit"
    }

    fn execute(&self, args: &[String], _reporter: &dyn Reporter) -> BuiltinResult {
        let code = args
            .first()
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(0);
        BuiltinResult::Exit(code)
    }
}
