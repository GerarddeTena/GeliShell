use crate::shell::reporter::Reporter;
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

        reporter.error("gerisabet not found in PATH");
        reporter.info("gerisabet is the GeliShell AI assistant companion binary.");
        reporter.info("");

        #[cfg(target_os = "windows")]
        {
            reporter.info("Install it by running:");
            reporter.info("  cargo build --release");
            reporter.info("  .\\install.ps1");
            reporter.info("");
            reporter.info("Or manually copy the binary:");
            let default_dir = default_install_dir();
            reporter.info(&format!(
                "  Copy target\\release\\gerisabet.exe → {}\\gerisabet.exe",
                default_dir
            ));
        }

        #[cfg(not(target_os = "windows"))]
        {
            reporter.info("Install it by running:");
            reporter.info("  cargo build --release");
            reporter.info("  ./install.sh");
            reporter.info("");
            reporter.info("Or manually copy the binary:");
            let default_dir = default_install_dir();
            reporter.info(&format!(
                "  cp target/release/gerisabet {}",
                default_dir
            ));
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
