pub mod config;
pub mod error;
pub mod platform;
pub mod result;

pub use config::ExecutionConfig;
pub use error::ExecutorError;
pub use result::{ExecTrace, ExecutionResult};

use crate::shell::reporter::Reporter;
use crate::shell::translator::subsystem::Subsystem;
use crate::t;
use std::sync::Arc;
use std::time::Instant;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, ChildStderr, ChildStdout};
use tokio::sync::Mutex as TokioMutex;
use tokio::task::JoinHandle;

// ══════════════════════════════════════════════════════════════
// Executor
// ══════════════════════════════════════════════════════════════

pub struct Executor {
    subsystem: Subsystem,
}

impl Executor {
    pub fn new(subsystem: Subsystem) -> Self {
        Self { subsystem }
    }

    pub fn requires_tty(command: &str, extra: &[String]) -> bool {
        let mut parts = command.split_whitespace().map(normalize_token);
        let Some(first) = parts.next() else {
            return false;
        };

        let entry = if matches!(first.as_str(), "sudo" | "env" | "command" | "nohup") {
            parts.next().unwrap_or(first)
        } else {
            first
        };

        if extra.iter().any(|e| e == entry.as_str()) {
            return true;
        }

        matches!(
            entry.as_str(),
            "nvim"
                | "nvim.exe"
                | "vim"
                | "vim.exe"
                | "vi"
                | "nano"
                | "less"
                | "more"
                | "man"
                | "top"
                | "htop"
                | "tmux"
                | "screen"
                | "gerisabet"
                | "gerisabet.exe"
        )
    }

    /// Ejecuta un comando nativo del subsistema.
    ///
    /// Hace streaming de stdout/stderr en tiempo real.
    /// Captura lo que indique `config`.
    ///
    /// # Errors
    /// - `ExecutorError::EmptyCommand`  — string vacío
    /// - `ExecutorError::SpawnFailed`   — el OS no pudo spawnear
    /// - `ExecutorError::KilledBySignal`— el proceso fue terminado
    /// - `ExecutorError::Timeout`       — superado el timeout
    pub async fn run(
        &self,
        command: &str,
        config: &ExecutionConfig,
        reporter: &dyn Reporter,
    ) -> Result<ExecutionResult, ExecutorError> {
        // ── Validación básica ─────────────────────────────────
        let command = command.trim();
        if command.is_empty() {
            return Err(ExecutorError::EmptyCommand);
        }

        reporter.info(&t!("executor.spawning", command = command, subsystem = self.subsystem));

        // ── Traza del comando ─────────────────────────────────
        let trace = config.capture_command_trace.then(|| ExecTrace {
            command: command.to_owned(),
            subsystem: self.subsystem.as_str().to_owned(),
        });

        // ── Inicia el timer si es necesario ───────────────────
        let start = config.capture_duration.then(Instant::now);

        // ── Construye y spawna el proceso ─────────────────────
        let mut cmd = platform::build_command(command, &self.subsystem);
        cmd.kill_on_drop(true);
        let interactive = Self::requires_tty(command, &config.extra_tty_commands);

        if interactive {
            reporter.info(&t!("executor.tty_mode"));
            cmd.stdin(std::process::Stdio::inherit());
            cmd.stdout(std::process::Stdio::inherit());
            cmd.stderr(std::process::Stdio::inherit());

            let child = cmd.spawn().map_err(ExecutorError::SpawnFailed)?;
            let exit_code = if let Some(secs) = config.timeout_secs {
                self.wait_with_timeout(child, secs).await?
            } else {
                self.wait(child).await?
            };
            let duration = start.map(|s| s.elapsed());

            reporter.info(&t!("executor.finished", code = exit_code));

            return Ok(ExecutionResult {
                exit_code,
                output: None,
                duration,
                trace,
            });
        }

        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

        let child = cmd.spawn().map_err(ExecutorError::SpawnFailed)?;
        let (exit_code, output) = self
            .stream_and_capture_with_timeout(child, config, reporter)
            .await?;

        // ── Duración ──────────────────────────────────────────
        let duration = start.map(|s| s.elapsed());

        reporter.info(&t!("executor.finished", code = exit_code));

        Ok(ExecutionResult {
            exit_code,
            output,
            duration,
            trace,
        })
    }

    // ──────────────────────────────────────────────────────────
    // Streaming híbrido — imprime en tiempo real + captura
    // ──────────────────────────────────────────────────────────

    async fn stream_and_capture_with_timeout(
        &self,
        mut child: Child,
        config: &ExecutionConfig,
        reporter: &dyn Reporter,
    ) -> Result<(i32, Option<String>), ExecutorError> {
        let captured = config
            .capture_output
            .then(|| Arc::new(TokioMutex::new(String::new())));

        let mut stdout_task = Self::spawn_stdout_task(child.stdout.take(), captured.clone());
        let mut stderr_task = Self::spawn_stderr_task(child.stderr.take(), captured.clone());

        let status = if let Some(secs) = config.timeout_secs {
            let timeout = tokio::time::Duration::from_secs(secs);
            match tokio::time::timeout(timeout, child.wait()).await {
                Ok(result) => result.map_err(ExecutorError::SpawnFailed)?,
                Err(_) => {
                    if let Err(error) = child.kill().await {
                        reporter.warn(&t!("executor.kill_failed", error = error));
                    }
                    if let Err(error) = child.wait().await {
                        reporter.warn(&t!("executor.wait_failed", error = error));
                    }

                    Self::finish_stream_task(&mut stdout_task).await?;
                    Self::finish_stream_task(&mut stderr_task).await?;
                    return Err(ExecutorError::Timeout(secs));
                }
            }
        } else {
            child.wait().await.map_err(ExecutorError::SpawnFailed)?
        };

        Self::finish_stream_task(&mut stdout_task).await?;
        Self::finish_stream_task(&mut stderr_task).await?;

        let output = match captured {
            Some(buffer) => Some(buffer.lock().await.clone()),
            None => None,
        };

        let exit_code = status.code().ok_or(ExecutorError::KilledBySignal)?;

        Ok((exit_code, output))
    }

    fn spawn_stdout_task(
        stdout: Option<ChildStdout>,
        captured: Option<Arc<TokioMutex<String>>>,
    ) -> Option<JoinHandle<Result<(), std::io::Error>>> {
        let stdout = stdout?;
        Some(tokio::spawn(async move {
            let mut reader = BufReader::new(stdout).lines();
            while let Some(line) = reader.next_line().await? {
                println!("{line}");
                if let Some(buffer) = &captured {
                    let mut locked = buffer.lock().await;
                    locked.push_str(&line);
                    locked.push('\n');
                }
            }
            Ok(())
        }))
    }

    fn spawn_stderr_task(
        stderr: Option<ChildStderr>,
        captured: Option<Arc<TokioMutex<String>>>,
    ) -> Option<JoinHandle<Result<(), std::io::Error>>> {
        let stderr = stderr?;
        Some(tokio::spawn(async move {
            let mut reader = BufReader::new(stderr).lines();
            while let Some(line) = reader.next_line().await? {
                eprintln!("{line}");
                if let Some(buffer) = &captured {
                    let mut locked = buffer.lock().await;
                    locked.push_str(&line);
                    locked.push('\n');
                }
            }
            Ok(())
        }))
    }

    async fn finish_stream_task(
        task: &mut Option<JoinHandle<Result<(), std::io::Error>>>,
    ) -> Result<(), ExecutorError> {
        let Some(handle) = task.take() else {
            return Ok(());
        };

        let joined = handle.await.map_err(|error| {
            ExecutorError::SpawnFailed(std::io::Error::other(format!(
                "stream task join failed: {error}"
            )))
        })?;

        joined.map_err(ExecutorError::SpawnFailed)
    }

    // ──────────────────────────────────────────────────────────
    // Espera al proceso
    // ──────────────────────────────────────────────────────────

    async fn wait(&self, mut child: Child) -> Result<i32, ExecutorError> {
        let status = child.wait().await.map_err(ExecutorError::SpawnFailed)?;

        status.code().ok_or(ExecutorError::KilledBySignal)
    }

    async fn wait_with_timeout(&self, mut child: Child, secs: u64) -> Result<i32, ExecutorError> {
        let timeout = tokio::time::Duration::from_secs(secs);

        tokio::time::timeout(timeout, child.wait())
            .await
            .map_err(|_| ExecutorError::Timeout(secs))?
            .map_err(ExecutorError::SpawnFailed)?
            .code()
            .ok_or(ExecutorError::KilledBySignal)
    }
}

fn normalize_token(token: &str) -> String {
    token
        .trim_matches('"')
        .trim_matches('\'')
        .to_ascii_lowercase()
}

// ══════════════════════════════════════════════════════════════
// Tests
// ══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shell::reporter::SilentReporter;
    use crate::shell::translator::subsystem::Subsystem;

    fn subsystem() -> Subsystem {
        #[cfg(target_os = "windows")]
        return Subsystem::PowerShell;
        #[cfg(not(target_os = "windows"))]
        return Subsystem::Bash;
    }

    // ──────────────────────────────────────────────────────────
    // Tests de ejecución básica
    // ──────────────────────────────────────────────────────────

    #[tokio::test]
    async fn executes_simple_command() {
        let executor = Executor::new(subsystem());
        let reporter = SilentReporter::new();
        let config = ExecutionConfig::minimal();

        #[cfg(not(target_os = "windows"))]
        let cmd = "echo hello";
        #[cfg(target_os = "windows")]
        let cmd = "echo hello";

        let result = executor.run(cmd, &config, &reporter).await.unwrap();
        assert!(result.success());
        assert_eq!(result.exit_code, 0);
    }

    #[tokio::test]
    async fn captures_output_when_configured() {
        let executor = Executor::new(subsystem());
        let reporter = SilentReporter::new();
        let config = ExecutionConfig::minimal().with_capture_output();

        #[cfg(not(target_os = "windows"))]
        let cmd = "echo captured_text";
        #[cfg(target_os = "windows")]
        let cmd = "echo captured_text";

        let result = executor.run(cmd, &config, &reporter).await.unwrap();
        assert!(result.output.is_some());
        assert!(result.output_or_empty().contains("captured_text"));
    }

    #[tokio::test]
    async fn no_output_captured_by_default() {
        let executor = Executor::new(subsystem());
        let reporter = SilentReporter::new();
        let config = ExecutionConfig::minimal();

        let result = executor
            .run("echo no_capture", &config, &reporter)
            .await
            .unwrap();

        assert!(result.output.is_none());
    }

    #[tokio::test]
    async fn captures_duration_when_configured() {
        let executor = Executor::new(subsystem());
        let reporter = SilentReporter::new();
        let config = ExecutionConfig::minimal().with_capture_duration();

        let result = executor
            .run("echo timing", &config, &reporter)
            .await
            .unwrap();

        assert!(result.duration.is_some());
        assert!(result.duration.unwrap().as_nanos() > 0);
    }

    #[tokio::test]
    async fn captures_trace_when_configured() {
        let executor = Executor::new(subsystem());
        let reporter = SilentReporter::new();
        let config = ExecutionConfig::minimal().with_capture_command_trace();

        let result = executor
            .run("echo trace", &config, &reporter)
            .await
            .unwrap();

        assert!(result.trace.is_some());
        let trace = result.trace.unwrap();
        assert_eq!(trace.command, "echo trace");
        assert!(!trace.subsystem.is_empty());
    }

    // ──────────────────────────────────────────────────────────
    // Tests de error
    // ──────────────────────────────────────────────────────────

    #[tokio::test]
    async fn returns_error_on_empty_command() {
        let executor = Executor::new(subsystem());
        let reporter = SilentReporter::new();
        let config = ExecutionConfig::minimal();

        let result = executor.run("   ", &config, &reporter).await;
        assert!(matches!(result, Err(ExecutorError::EmptyCommand)));
    }

    #[tokio::test]
    async fn nonzero_exit_code_on_failure() {
        let executor = Executor::new(subsystem());
        let reporter = SilentReporter::new();
        let config = ExecutionConfig::minimal();

        #[cfg(not(target_os = "windows"))]
        let cmd = "exit 1";
        #[cfg(target_os = "windows")]
        let cmd = "exit 1";

        let result = executor.run(cmd, &config, &reporter).await.unwrap();
        assert!(!result.success());
        assert_ne!(result.exit_code, 0);
    }

    #[tokio::test]
    async fn timeout_returns_error() {
        let executor = Executor::new(subsystem());
        let reporter = SilentReporter::new();
        let config = ExecutionConfig::minimal().with_timeout(1);

        #[cfg(not(target_os = "windows"))]
        let cmd = "sleep 10";
        #[cfg(target_os = "windows")]
        let cmd = "Start-Sleep 10";

        let result = executor.run(cmd, &config, &reporter).await;
        assert!(matches!(result, Err(ExecutorError::Timeout(1))));
    }

    // ──────────────────────────────────────────────────────────
    // Tests de config
    // ──────────────────────────────────────────────────────────

    #[test]
    fn full_config_has_all_options_enabled() {
        let config = ExecutionConfig::full();
        assert!(config.capture_output);
        assert!(config.capture_duration);
        assert!(config.capture_command_trace);
    }

    #[test]
    fn minimal_config_has_no_options() {
        let config = ExecutionConfig::minimal();
        assert!(!config.capture_output);
        assert!(!config.capture_duration);
        assert!(!config.capture_command_trace);
        assert!(config.timeout_secs.is_none());
    }

    #[test]
    fn builder_activates_only_requested_options() {
        let config = ExecutionConfig::minimal()
            .with_capture_duration()
            .with_timeout(30);

        assert!(!config.capture_output);
        assert!(config.capture_duration);
        assert!(!config.capture_command_trace);
        assert_eq!(config.timeout_secs, Some(30));
    }

    #[test]
    fn detects_nvim_as_tty_command() {
        assert!(Executor::requires_tty("nvim Cargo.toml", &[]));
        assert!(Executor::requires_tty("\"nvim\" src/main.rs", &[]));
        assert!(Executor::requires_tty("sudo nvim Cargo.toml", &[]));
    }

    #[test]
    fn ignores_non_tty_command() {
        assert!(!Executor::requires_tty("echo hello", &[]));
        assert!(!Executor::requires_tty("ls -la", &[]));
    }

    #[test]
    fn extra_tty_commands_extend_built_in_list() {
        let extra = vec!["lazygit".to_owned(), "helix".to_owned()];
        assert!(Executor::requires_tty("lazygit", &extra));
        assert!(Executor::requires_tty("helix .", &extra));
        // Built-in list still works
        assert!(Executor::requires_tty("nvim", &extra));
        // Non-listed commands are unaffected
        assert!(!Executor::requires_tty("cargo build", &extra));
    }
}
