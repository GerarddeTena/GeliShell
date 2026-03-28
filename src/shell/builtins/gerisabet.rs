use crate::shell::reporter::Reporter;
use crate::t;
use super::{Builtin, BuiltinResult};

// ══════════════════════════════════════════════════════════════
// GerisabetBuiltin
// ══════════════════════════════════════════════════════════════

/// Intercepta `gerisabet [args]` antes del pipeline de traducción.
///
/// Si el binario está disponible en PATH o en el directorio de instalación
/// por defecto, retorna `NotABuiltin` para que el executor lo maneje como
/// un proceso TTY con stdio heredado.
///
/// Si no está disponible, muestra instrucciones claras en lugar del
/// críptico "program not found".
pub struct GerisabetBuiltin;

impl Builtin for GerisabetBuiltin {
    fn name(&self) -> &'static str {
        "gerisabet"
    }

    fn execute(&self, _args: &[String], reporter: &dyn Reporter) -> BuiltinResult {
        if is_gerisabet_available() {
            // Binary found — let the executor handle it with TTY mode
            return BuiltinResult::NotABuiltin;
        }

        reporter.error(&t!("builtin.gerisabet.not_found"));
        reporter.info(&t!("builtin.gerisabet.description"));
        reporter.info("");

        #[cfg(target_os = "windows")]
        {
            reporter.info(&t!("builtin.gerisabet.install_hint"));
            reporter.info(&t!("builtin.gerisabet.install_step1"));
            reporter.info(&t!("builtin.gerisabet.install_step2_windows"));
            reporter.info("");
            reporter.info(&t!("builtin.gerisabet.manual_copy"));
            let default_dir = default_install_dir();
            reporter.info(&t!("builtin.gerisabet.copy_hint_windows", dir = default_dir));
        }

        #[cfg(not(target_os = "windows"))]
        {
            reporter.info(&t!("builtin.gerisabet.install_hint"));
            reporter.info(&t!("builtin.gerisabet.install_step1"));
            reporter.info(&t!("builtin.gerisabet.install_step2_unix"));
            reporter.info("");
            reporter.info(&t!("builtin.gerisabet.manual_copy"));
            let default_dir = default_install_dir();
            reporter.info(&t!("builtin.gerisabet.copy_hint_unix", dir = default_dir));
        }

        BuiltinResult::Handled
    }
}

// ── Helpers ───────────────────────────────────────────────────

/// Comprueba si `gerisabet` (o `gerisabet.exe` en Windows) está disponible
/// en PATH o en el directorio de instalación por defecto de GeliShell.
fn is_gerisabet_available() -> bool {
    // 1. Check PATH entries
    let path_var = std::env::var("PATH").unwrap_or_default();

    #[cfg(target_os = "windows")]
    let sep = ';';
    #[cfg(not(target_os = "windows"))]
    let sep = ':';

    for dir in path_var.split(sep) {
        let base = std::path::Path::new(dir);

        #[cfg(target_os = "windows")]
        if base.join("gerisabet.exe").exists() || base.join("gerisabet").exists() {
            return true;
        }

        #[cfg(not(target_os = "windows"))]
        if base.join("gerisabet").exists() {
            return true;
        }
    }

    // 2. Check the default GeliShell install location even if not in PATH
    let default_dir = default_install_dir();
    let default_path = std::path::Path::new(&default_dir);

    #[cfg(target_os = "windows")]
    return default_path.join("gerisabet.exe").exists();

    #[cfg(not(target_os = "windows"))]
    default_path.join("gerisabet").exists()
}

/// Directorio de instalación por defecto según plataforma.
fn default_install_dir() -> String {
    #[cfg(target_os = "windows")]
    {
        let home = std::env::var("USERPROFILE").unwrap_or_else(|_| "C:\\Users\\default".to_owned());
        format!("{}\\.local\\bin", home)
    }

    #[cfg(not(target_os = "windows"))]
    {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/user".to_owned());
        format!("{}/.local/bin", home)
    }
}
