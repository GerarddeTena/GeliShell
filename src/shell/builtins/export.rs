use super::{Builtin, BuiltinResult};
use crate::shell::reporter::Reporter;
use crate::t;

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
                reporter.info(&t!("builtin.export.set", key = key, value = value));
            } else {
                // Sin valor — exporta la variable si ya existe
                if let Ok(val) = std::env::var(arg) {
                    reporter.info(&t!("builtin.export.set", key = arg, value = val));
                } else {
                    reporter.warn(&t!("builtin.export.not_found", arg = arg));
                }
            }
        }
        BuiltinResult::Handled
    }
}
