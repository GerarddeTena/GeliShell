use std::time::Duration;

/// Traza del comando ejecutado — solo presente si
/// `ExecutionConfig::capture_command_trace` = true
#[derive(Debug, Clone)]
pub struct ExecTrace {
    /// El string exacto que se pasó al proceso
    pub command:   String,
    /// El subsistema que lo ejecutó
    pub subsystem: String,
}

/// Resultado de la ejecución de un proceso.
///
/// `exit_code` siempre está presente.
/// El resto son `Option` controlados por `ExecutionConfig`.
#[derive(Debug)]
pub struct ExecutionResult {
    /// Exit code del proceso — 0 = éxito, != 0 = error
    /// Siempre presente independientemente de la config
    pub exit_code: i32,

    /// stdout + stderr combinados si capture_output = true
    pub output: Option<String>,

    /// Duración total si capture_duration = true
    pub duration: Option<Duration>,

    /// Traza del comando si capture_command_trace = true
    pub trace: Option<ExecTrace>,
}

impl ExecutionResult {
    /// true si el proceso terminó con exit code 0
    pub fn success(&self) -> bool {
        self.exit_code == 0
    }

    /// Devuelve el output capturado o string vacío
    pub fn output_or_empty(&self) -> &str {
        self.output.as_deref().unwrap_or("")
    }
}