pub mod config;
pub mod error;
pub mod platform;
pub mod result;

pub use config::ExecutionConfig;
pub use error::ExecutorError;
pub use result::{ExecTrace, ExecutionResult};

use crate::shell::reporter::Reporter;
use crate::shell::translator::subsystem::Subsystem;
use std::time::Instant;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Child;

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
        command:  &str,
        config:   &ExecutionConfig,
        reporter: &dyn Reporter,
    ) -> Result<ExecutionResult, ExecutorError> {
        // ── Validación básica ─────────────────────────────────
        let command = command.trim();
        if command.is_empty() {
            return Err(ExecutorError::EmptyCommand);
        }

        reporter.info(&format!(
            "executor: spawning '{}' via {}",
            command, self.subsystem
        ));

        // ── Traza del comando ─────────────────────────────────
        let trace = config.capture_command_trace.then(|| ExecTrace {
            command:   command.to_owned(),
            subsystem: self.subsystem.as_str().to_owned(),
        });

        // ── Inicia el timer si es necesario ───────────────────
        let start = config.capture_duration.then(Instant::now);

        // ── Construye y spawna el proceso ─────────────────────
        let mut cmd = platform::build_command(command, &self.subsystem);
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

        let mut child = cmd.spawn().map_err(ExecutorError::SpawnFailed)?;

        // ── Streaming de stdout y stderr ──────────────────────
        let output = self
            .stream_and_capture(&mut child, config, reporter)
            .await?;

        // ── Espera al proceso con timeout opcional ────────────
        let exit_code = if let Some(secs) = config.timeout_secs {
            self.wait_with_timeout(child, secs).await?
        } else {
            self.wait(child).await?
        };

        // ── Duración ──────────────────────────────────────────
        let duration = start.map(|s| s.elapsed());

        reporter.info(&format!(
            "executor: finished with exit code {exit_code}"
        ));

        Ok(ExecutionResult { exit_code, output, duration, trace })
    }

    // ──────────────────────────────────────────────────────────
    // Streaming híbrido — imprime en tiempo real + captura
    // ──────────────────────────────────────────────────────────

    async fn stream_and_capture(
        &self,
        child:    &mut Child,
        config:   &ExecutionConfig,
        _reporter: &dyn Reporter,
    ) -> Result<Option<String>, ExecutorError> {
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        // Buffer acumulador — solo se usa si capture_output = true
        let mut captured = config.capture_output.then(String::new);

        // ── Streaming de stdout ───────────────────────────────
        if let Some(stdout) = stdout {
            let mut reader = BufReader::new(stdout).lines();
            while let Some(line) = reader
                .next_line()
                .await
                .map_err(ExecutorError::SpawnFailed)?
            {
                // Imprime en tiempo real — siempre
                println!("{line}");

                // Captura si config lo indica
                if let Some(ref mut buf) = captured {
                    buf.push_str(&line);
                    buf.push('\n');
                }
            }
        }

        // ── Streaming de stderr ───────────────────────────────
        if let Some(stderr) = stderr {
            let mut reader = BufReader::new(stderr).lines();
            while let Some(line) = reader
                .next_line()
                .await
                .map_err(ExecutorError::SpawnFailed)?
            {
                // stderr va a eprintln — siempre visible
                eprintln!("{line}");

                if let Some(ref mut buf) = captured {
                    buf.push_str(&line);
                    buf.push('\n');
                }
            }
        }

        Ok(captured)
    }

    // ──────────────────────────────────────────────────────────
    // Espera al proceso
    // ──────────────────────────────────────────────────────────

    async fn wait(
        &self,
        mut child: Child,
    ) -> Result<i32, ExecutorError> {
        let status = child
            .wait()
            .await
            .map_err(ExecutorError::SpawnFailed)?;

        status
            .code()
            .ok_or(ExecutorError::KilledBySignal)
    }

    async fn wait_with_timeout(
        &self,
        mut child: Child,
        secs:      u64,
    ) -> Result<i32, ExecutorError> {
        let timeout = tokio::time::Duration::from_secs(secs);

        tokio::time::timeout(timeout, child.wait())
            .await
            .map_err(|_| ExecutorError::Timeout(secs))?
            .map_err(ExecutorError::SpawnFailed)?
            .code()
            .ok_or(ExecutorError::KilledBySignal)
    }
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
        let config   = ExecutionConfig::minimal();

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
        let config   = ExecutionConfig::minimal().with_capture_output();

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
        let config   = ExecutionConfig::minimal();

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
        let config   = ExecutionConfig::minimal().with_capture_duration();

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
        let config   = ExecutionConfig::minimal().with_capture_command_trace();

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
        let config   = ExecutionConfig::minimal();

        let result = executor.run("   ", &config, &reporter).await;
        assert!(matches!(result, Err(ExecutorError::EmptyCommand)));
    }

    #[tokio::test]
    async fn nonzero_exit_code_on_failure() {
        let executor = Executor::new(subsystem());
        let reporter = SilentReporter::new();
        let config   = ExecutionConfig::minimal();

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
        let config   = ExecutionConfig::minimal().with_timeout(1);

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
}