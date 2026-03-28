use super::{Builtin, BuiltinResult};
use crate::shell::reporter::Reporter;
use crate::t;

pub struct SourceBuiltin;

impl Builtin for SourceBuiltin {
    fn name(&self) -> &'static str {
        "source"
    }

    fn execute(&self, args: &[String], reporter: &dyn Reporter) -> BuiltinResult {
        let Some(path) = args.first() else {
            reporter.error(&t!("builtin.source.missing_arg"));
            return BuiltinResult::Handled;
        };
        // Placeholder — la ejecución de scripts se implementará
        // cuando el scripting engine esté listo
        reporter.warn(&t!("builtin.source.not_available", path = path));
        BuiltinResult::Handled
    }
}
