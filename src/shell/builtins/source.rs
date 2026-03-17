use super::{Builtin, BuiltinResult};
use crate::shell::reporter::Reporter;

pub struct SourceBuiltin;

impl Builtin for SourceBuiltin {
    fn name(&self) -> &'static str {
        "source"
    }

    fn execute(&self, args: &[String], reporter: &dyn Reporter) -> BuiltinResult {
        let Some(path) = args.first() else {
            reporter.error("source: missing file argument");
            return BuiltinResult::Handled;
        };
        // Placeholder — la ejecución de scripts se implementará
        // cuando el scripting engine esté listo
        reporter.warn(&format!(
            "source: '{path}' — scripting engine not yet available"
        ));
        BuiltinResult::Handled
    }
}
