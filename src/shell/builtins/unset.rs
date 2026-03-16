use super::{Builtin, BuiltinResult};
use crate::shell::reporter::Reporter;

pub struct UnsetBuiltin;

impl Builtin for UnsetBuiltin {
    fn name(&self) -> &'static str { "unset" }

    fn execute(&self, args: &[String], reporter: &dyn Reporter) -> BuiltinResult {
        for arg in args {
            unsafe { std::env::remove_var(arg); }
            reporter.info(&format!("unset: removed '{arg}'"));
        }
        BuiltinResult::Handled
    }
}