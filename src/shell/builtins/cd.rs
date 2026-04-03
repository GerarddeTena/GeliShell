use super::{Builtin, BuiltinResult};
use crate::shell::reporter::Reporter;
use crate::t;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub struct CdBuiltin {
    /// Shared previous-directory state with GJumpBuiltin.
    /// Eliminates the need to set OLDPWD in the process environment.
    oldpwd: Arc<Mutex<Option<PathBuf>>>,
}

impl CdBuiltin {
    pub fn new(oldpwd: Arc<Mutex<Option<PathBuf>>>) -> Self {
        Self { oldpwd }
    }
}

impl Builtin for CdBuiltin {
    fn name(&self) -> &'static str {
        "cd"
    }

    fn execute(&self, args: &[String], reporter: &dyn Reporter) -> BuiltinResult {
        let target = args.first().map(|s| s.as_str()).unwrap_or("~");

        let path = if target == "~" || target == "$HOME" {
            home_dir()
        } else if target == "-" {
            // Read previous dir from shared application state instead of env.
            self.oldpwd
                .lock()
                .ok()
                .and_then(|g| g.clone())
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_else(|| ".".to_owned())
        } else {
            target.to_owned()
        };

        // Snapshot current dir into shared state before changing it.
        if let Ok(current) = std::env::current_dir() {
            if let Ok(mut guard) = self.oldpwd.lock() {
                *guard = Some(current);
            }
        }

        match std::env::set_current_dir(&path) {
            Ok(_) => {
                // SAFETY: `set_var("PWD")` is required so child processes inherit
                // the correct POSIX `$PWD` (including symlink-preserved paths).
                // Builtins execute synchronously with no intervening `.await`
                // points; the only other async tasks active are signal handlers
                // which do not read the process environment.
                unsafe {
                    if let Ok(new) = std::env::current_dir() {
                        std::env::set_var("PWD", new.to_string_lossy().as_ref());
                    }
                }
                BuiltinResult::Handled
            }
            Err(e) => {
                reporter.error(&t!("builtin.cd.error", path = path, error = e));
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
