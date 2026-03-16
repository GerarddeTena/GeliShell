use super::{Builtin, BuiltinResult};
use crate::shell::reporter::Reporter;

pub struct CdBuiltin;

impl Builtin for CdBuiltin {
    fn name(&self) -> &'static str { "cd" }

    fn execute(&self, args: &[String], reporter: &dyn Reporter) -> BuiltinResult {
        let target = args.first()
            .map(|s| s.as_str())
            .unwrap_or("~");

        let path = if target == "~" || target == "$HOME" {
            std::env::var("HOME")
                .unwrap_or_else(|_| "/".to_owned())
        } else if target == "-" {
            std::env::var("OLDPWD")
                .unwrap_or_else(|_| ".".to_owned())
        } else {
            target.to_owned()
        };

        // Guarda el directorio actual en OLDPWD antes de cambiar
        if let Ok(current) = std::env::current_dir() {
            unsafe { std::env::set_var("OLDPWD", current.to_string_lossy().as_ref()); }
        }

        match std::env::set_current_dir(&path) {
            Ok(_) => unsafe {
                // Actualiza PWD
                if let Ok(new) = std::env::current_dir() {
                    std::env::set_var("PWD", new.to_string_lossy().as_ref());
                }
                BuiltinResult::Handled
            }
            Err(e) => {
                reporter.error(&format!("cd: {path}: {e}"));
                BuiltinResult::Handled
            }
        }
    }
}