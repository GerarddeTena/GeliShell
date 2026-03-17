use super::{Builtin, BuiltinResult};
use crate::shell::reporter::Reporter;

pub struct ExportBuiltin;

impl Builtin for ExportBuiltin {
    fn name(&self) -> &'static str {
        "export"
    }

    fn execute(&self, args: &[String], reporter: &dyn Reporter) -> BuiltinResult {
        for arg in args {
            if let Some((key, value)) = arg.split_once('=') {
                unsafe {
                    std::env::set_var(key.trim(), value.trim());
                }
                reporter.info(&format!("export: {key}={value}"));
            } else {
                // Sin valor — exporta la variable si ya existe
                if let Ok(val) = std::env::var(arg) {
                    reporter.info(&format!("export: {arg}={val}"));
                } else {
                    reporter.warn(&format!("export: '{arg}' not found"));
                }
            }
        }
        BuiltinResult::Handled
    }
}
