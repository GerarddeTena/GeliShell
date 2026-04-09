use crate::shell::translator::subsystem::Subsystem;
use tokio::process::Command;
#[cfg(target_os = "windows")]
use std::sync::OnceLock;

/// Resuelve el ejecutable correcto de PowerShell escaneando PATH.
///
/// Prefiere `pwsh` (PowerShell 7+) sobre `powershell` (Windows PowerShell 5).
/// Fallback a la ruta absoluta conocida de Windows PowerShell 5 si ninguno
/// aparece en PATH. No lanza procesos — solo comprueba existencia de archivos.
#[cfg(target_os = "windows")]
fn resolve_powershell_exe() -> &'static str {
    static EXE: OnceLock<String> = OnceLock::new();
    EXE.get_or_init(|| {
        let path_var = std::env::var("PATH").unwrap_or_default();
        for dir in path_var.split(';') {
            let base = std::path::Path::new(dir);
            if base.join("pwsh.exe").exists() || base.join("pwsh").exists() {
                return "pwsh".to_owned();
            }
            if base.join("powershell.exe").exists() || base.join("powershell").exists() {
                return "powershell".to_owned();
            }
        }
        // Known absolute fallback — always present on Windows 10/11
        r"C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe".to_owned()
    })
    .as_str()
}

/// Construye el Command de tokio apropiado para el subsistema
/// y la plataforma actual.
///
/// En Unix envuelve en `sh -c` para pipelines y operadores.
/// En Windows usa `cmd /C` o `powershell/pwsh -Command` según subsistema.
///
/// Para PowerShell se fuerza UTF-8 en stdout/stderr antes de ejecutar el
/// comando del usuario. Sin esto, cualquier carácter no-ASCII en el output
/// (nombres de usuario, rutas, valores de entorno) produce bytes inválidos
/// para el lector UTF-8 del executor.
pub fn build_command(command: &str, subsystem: &Subsystem) -> Command {
    match subsystem {
        Subsystem::PowerShell => {
            #[cfg(target_os = "windows")]
            let exe = resolve_powershell_exe();
            #[cfg(not(target_os = "windows"))]
            let exe = "pwsh";

            // Force UTF-8 on both Console encoding and $OutputEncoding so that
            // piped stdout/stderr is always valid UTF-8 for BufReader::lines().
            let utf8_cmd = format!(
                "[Console]::OutputEncoding = [System.Text.Encoding]::UTF8; \
                 $OutputEncoding = [System.Text.Encoding]::UTF8; \
                 {}",
                command
            );

            let mut cmd = Command::new(exe);
            cmd.args(["-NoProfile", "-NonInteractive", "-Command", &utf8_cmd]);
            cmd.env("GELISHELL_ACTIVE", "1");
            cmd
        }
        Subsystem::Cmd => {
            let mut cmd = Command::new("cmd");
            cmd.args(["/C", command]);
            cmd.env("GELISHELL_ACTIVE", "1");
            cmd
        }
        // Bash, Zsh, Fish — todos entienden sh -c en Unix
        // En Windows con Git Bash o WSL también funciona
        Subsystem::Bash | Subsystem::Zsh | Subsystem::Fish => {
            #[cfg(target_os = "windows")]
            {
                // Git Bash / WSL path
                let mut cmd = Command::new("sh");
                cmd.args(["-c", command]);
                cmd.env("GELISHELL_ACTIVE", "1");
                cmd
            }
            #[cfg(not(target_os = "windows"))]
            {
                let shell = match subsystem {
                    Subsystem::Zsh => "zsh",
                    Subsystem::Fish => "fish",
                    _ => "bash",
                };
                let mut cmd = Command::new(shell);
                cmd.args(["-c", command]);
                cmd.env("GELISHELL_ACTIVE", "1");
                cmd
            }
        }
    }
}
