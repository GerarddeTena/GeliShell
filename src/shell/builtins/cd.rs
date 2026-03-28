use super::{Builtin, BuiltinResult};
use crate::shell::reporter::Reporter;

pub struct CdBuiltin;

impl Builtin for CdBuiltin {
    fn name(&self) -> &'static str {
        "cd"
    }

    fn execute(&self, args: &[String], reporter: &dyn Reporter) -> BuiltinResult {
        let target = args.first().map(|s| s.as_str()).unwrap_or("~");

        let path = if target == "~" || target == "$HOME" {
            home_dir()
        } else if target == "-" {
            std::env::var("OLDPWD").unwrap_or_else(|_| ".".to_owned())
        } else {
            target.to_owned()
        };

        if let Ok(current) = std::env::current_dir() {
            // SAFETY: the REPL loop is single-threaded at this point;
            // no concurrent thread reads OLDPWD while we write it.
            unsafe {
                std::env::set_var("OLDPWD", current.to_string_lossy().as_ref());
            }
        }

        match std::env::set_current_dir(&path) {
            Ok(_) => unsafe {
                // SAFETY: same single-threaded REPL guarantee as OLDPWD above.
                if let Ok(new) = std::env::current_dir() {
                    std::env::set_var("PWD", new.to_string_lossy().as_ref());
                }
                BuiltinResult::Handled
            },
            Err(e) => {
                reporter.error(&format!("cd: {path}: {e}"));
                BuiltinResult::Handled
            }
        }
    }
}

/// Devuelve el directorio home del usuario de forma cross-platform.
/// Windows: USERPROFILE (con fallback a HOME para entornos tipo WSL/Git Bash).
/// Unix:    HOME.
fn home_dir() -> String {
    #[cfg(target_os = "windows")]
    {
        std::env::var("USERPROFILE")
            .or_else(|_| std::env::var("HOME"))
            .unwrap_or_else(|_| "C:\\Users\\Default".to_owned())
    }
    #[cfg(not(target_os = "windows"))]
    {
        std::env::var("HOME").unwrap_or_else(|_| "/".to_owned())
    }
}
