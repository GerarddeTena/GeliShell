use crate::shell::translator::subsystem::Subsystem;
use tokio::process::Command;

/// Construye el Command de tokio apropiado para el subsistema
/// y la plataforma actual.
///
/// En Unix envuelve en `sh -c` para pipelines y operadores.
/// En Windows usa `cmd /C` o `powershell -Command` según subsistema.
pub fn build_command(command: &str, subsystem: &Subsystem) -> Command {
    match subsystem {
        Subsystem::PowerShell => {
            let mut cmd = Command::new("powershell");
            cmd.args(["-NoProfile", "-NonInteractive", "-Command", command]);
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
