/// Contrato de presentación de mensajes del sistema.
/// Cualquier módulo que necesite emitir output implementa
/// este trait en vez de llamar a eprintln! directamente.

pub trait Reporter: Send + Sync {
    fn warn(&self, message: &str);
    fn error(&self, message: &str);
    fn info(&self, message: &str);

    // Helper con formato — no sobreescribir
    fn warn_fmt(&self, args: std::fmt::Arguments<'_>) {
        self.warn(&args.to_string());
    }
    fn error_fmt(&self, args: std::fmt::Arguments<'_>) {
        self.error(&args.to_string());
    }
    fn info_fmt(&self, args: std::fmt::Arguments<'_>) {
        self.info(&args.to_string());
    }
}

// ══════════════════════════════════════════════════════════════
// Macros de conveniencia — evitan format!() en hot paths
// (mem-avoid-format de las Rust skills)
// ══════════════════════════════════════════════════════════════

/// Emite un warning a través del reporter
#[macro_export]
macro_rules! report_warn {
    ($reporter:expr, $($arg:tt)*) => {
        $reporter.warn_fmt(format_args!($($arg)*))
    };
}

/// Emite un error a través del reporter
#[macro_export]
macro_rules! report_error {
    ($reporter:expr, $($arg:tt)*) => {
        $reporter.error_fmt(format_args!($($arg)*))
    };
}

/// Emite info a través del reporter
#[macro_export]
macro_rules! report_info {
    ($reporter:expr, $($arg:tt)*) => {
        $reporter.info_fmt(format_args!($($arg)*))
    };
}

// ══════════════════════════════════════════════════════════════
// StderrReporter — producción
// ══════════════════════════════════════════════════════════════

/// Reporter de producción — escribe a stderr con prefijos visuales
pub struct StderrReporter;

impl StderrReporter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for StderrReporter {
    fn default() -> Self {
        Self::new()
    }
}

impl Reporter for StderrReporter {
    fn warn(&self, message: &str) {
        eprintln!("⚠  {message}");
    }
    fn error(&self, message: &str) {
        eprintln!("✖  {message}");
    }
    fn info(&self, message: &str) {
        eprintln!("ℹ  {message}");
    }
}

// ══════════════════════════════════════════════════════════════
// SilentReporter — tests unitarios
// ══════════════════════════════════════════════════════════════

/// Descarta todo output — para tests que no deben contaminar stdout/stderr
pub struct SilentReporter;

impl SilentReporter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SilentReporter {
    fn default() -> Self {
        Self::new()
    }
}

impl Reporter for SilentReporter {
    fn warn(&self, _message: &str) {}
    fn error(&self, _message: &str) {}
    fn info(&self, _message: &str) {}
}

// ══════════════════════════════════════════════════════════════
// BufferedReporter — tests de integración
// ══════════════════════════════════════════════════════════════

use std::sync::{Arc, Mutex};

/// Acumula mensajes en memoria — permite assertions en tests
///
/// # Example
/// ```rust
/// use geli_shell::{BufferedReporter, Reporter};
///
/// let reporter = BufferedReporter::new();
/// reporter.warn("algo salió mal");
/// assert!(reporter.has_warnings());
/// assert_eq!(reporter.warnings()[0], "algo salió mal");
/// ```
pub struct BufferedReporter {
    warnings: Arc<Mutex<Vec<String>>>,
    errors: Arc<Mutex<Vec<String>>>,
    infos: Arc<Mutex<Vec<String>>>,
}

impl BufferedReporter {
    pub fn new() -> Self {
        Self {
            warnings: Arc::new(Mutex::new(Vec::new())),
            errors: Arc::new(Mutex::new(Vec::new())),
            infos: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn warnings(&self) -> Vec<String> {
        self.warnings.lock().unwrap().clone()
    }

    pub fn errors(&self) -> Vec<String> {
        self.errors.lock().unwrap().clone()
    }

    pub fn infos(&self) -> Vec<String> {
        self.infos.lock().unwrap().clone()
    }

    pub fn has_warnings(&self) -> bool {
        !self.warnings.lock().unwrap().is_empty()
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.lock().unwrap().is_empty()
    }

    /// Vacía todos los buffers — útil entre assertions en el mismo test
    pub fn clear(&self) {
        self.warnings.lock().unwrap().clear();
        self.errors.lock().unwrap().clear();
        self.infos.lock().unwrap().clear();
    }

    /// Total de mensajes acumulados de cualquier nivel
    pub fn total(&self) -> usize {
        self.warnings.lock().unwrap().len()
            + self.errors.lock().unwrap().len()
            + self.infos.lock().unwrap().len()
    }
}

impl Default for BufferedReporter {
    fn default() -> Self {
        Self::new()
    }
}

impl Reporter for BufferedReporter {
    fn warn(&self, message: &str) {
        self.warnings.lock().unwrap().push(message.to_owned());
    }
    fn error(&self, message: &str) {
        self.errors.lock().unwrap().push(message.to_owned());
    }
    fn info(&self, message: &str) {
        self.infos.lock().unwrap().push(message.to_owned());
    }
}
