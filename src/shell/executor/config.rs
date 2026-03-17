/// Configuración del executor — accesible desde los settings
/// de la shell pero no modificable desde el output.
///
/// Todos los campos excepto `exit_code` son opt-in:
/// el usuario activa lo que necesita sin overhead innecesario.
#[derive(Debug, Clone)]
pub struct ExecutionConfig {
    /// Captura stdout+stderr además de hacer streaming.
    /// Por defecto false — solo streaming en tiempo real.
    pub capture_output: bool,

    /// Mide y registra la duración de ejecución.
    /// Por defecto false.
    pub capture_duration: bool,

    /// Registra el comando exacto ejecutado y el subsistema.
    /// Por defecto false.
    pub capture_command_trace: bool,

    /// Timeout en segundos — None significa sin límite.
    /// Por defecto None.
    pub timeout_secs: Option<u64>,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            capture_output: false,
            capture_duration: false,
            capture_command_trace: false,
            timeout_secs: None,
        }
    }
}

impl ExecutionConfig {
    /// Config mínima — solo exit code, sin overhead
    pub fn minimal() -> Self {
        Self::default()
    }

    /// Config completa — todas las opciones activas
    /// Útil para preprod y debugging
    pub fn full() -> Self {
        Self {
            capture_output: true,
            capture_duration: true,
            capture_command_trace: true,
            timeout_secs: None,
        }
    }

    /// Builder — permite activar solo lo necesario
    pub fn with_capture_output(mut self) -> Self {
        self.capture_output = true;
        self
    }

    pub fn with_capture_duration(mut self) -> Self {
        self.capture_duration = true;
        self
    }

    pub fn with_capture_command_trace(mut self) -> Self {
        self.capture_command_trace = true;
        self
    }

    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.timeout_secs = Some(secs);
        self
    }
}
