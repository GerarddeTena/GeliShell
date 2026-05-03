use crate::shell::config::ReporterLevel;

/// Contrato de presentación de mensajes del sistema.
/// Cualquier módulo que necesite emitir output implementa
/// este trait en vez de llamar a eprintln! directamente.
pub trait Reporter: Send + Sync {
    fn warn(&self, message: &str);
    fn error(&self, message: &str);
    fn info(&self, message: &str);
    fn raw_stdout(&self, message: &str);
    fn raw_stderr(&self, message: &str);

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
pub struct StderrReporter {
    level: ReporterLevel,
}

impl StderrReporter {
    pub fn new(level: ReporterLevel) -> Self {
        Self { level }
    }

    pub fn level(&self) -> ReporterLevel {
        self.level
    }
}

impl Default for StderrReporter {
    fn default() -> Self {
        Self::new(ReporterLevel::Error)
    }
}

impl Reporter for StderrReporter {
    fn warn(&self, message: &str) {
        if !self.level.allows_warn() {
            return;
        }
        // Yellow [ 󰀦 ]
        eprintln!("\x1b[38;5;220m[ 󰀦 ]\x1b[0m  {message}");
    }
    fn error(&self, message: &str) {
        // Red [ 󰅖 ]
        eprintln!("\x1b[38;5;196m[ 󰅖 ]\x1b[0m  {message}");
    }
    fn info(&self, message: &str) {
        if !self.level.allows_info() {
            return;
        }
        // Blue/Cyan [ 󰋼 ]
        eprintln!("\x1b[38;5;39m[ 󰋼 ]\x1b[0m  {message}");
    }

    fn raw_stdout(&self, message: &str) {
        println!("{message}");
    }

    fn raw_stderr(&self, message: &str) {
        eprintln!("{message}");
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
    fn raw_stdout(&self, _message: &str) {}
    fn raw_stderr(&self, _message: &str) {}
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
    stdout: Arc<Mutex<Vec<String>>>,
    stderr: Arc<Mutex<Vec<String>>>,
}

impl BufferedReporter {
    pub fn new() -> Self {
        Self {
            warnings: Arc::new(Mutex::new(Vec::new())),
            errors: Arc::new(Mutex::new(Vec::new())),
            infos: Arc::new(Mutex::new(Vec::new())),
            stdout: Arc::new(Mutex::new(Vec::new())),
            stderr: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn warnings(&self) -> Vec<String> {
        self.warnings
            .lock()
            .expect("BufferedReporter lock poisoned")
            .clone()
    }

    pub fn errors(&self) -> Vec<String> {
        self.errors
            .lock()
            .expect("BufferedReporter lock poisoned")
            .clone()
    }

    pub fn infos(&self) -> Vec<String> {
        self.infos
            .lock()
            .expect("BufferedReporter lock poisoned")
            .clone()
    }

    pub fn stdout_lines(&self) -> Vec<String> {
        self.stdout
            .lock()
            .expect("BufferedReporter lock poisoned")
            .clone()
    }

    pub fn stderr_lines(&self) -> Vec<String> {
        self.stderr
            .lock()
            .expect("BufferedReporter lock poisoned")
            .clone()
    }

    pub fn has_warnings(&self) -> bool {
        !self
            .warnings
            .lock()
            .expect("BufferedReporter lock poisoned")
            .is_empty()
    }

    pub fn has_errors(&self) -> bool {
        !self
            .errors
            .lock()
            .expect("BufferedReporter lock poisoned")
            .is_empty()
    }

    /// Vacía todos los buffers — útil entre assertions en el mismo test
    pub fn clear(&self) {
        self.warnings
            .lock()
            .expect("BufferedReporter lock poisoned")
            .clear();
        self.errors
            .lock()
            .expect("BufferedReporter lock poisoned")
            .clear();
        self.infos
            .lock()
            .expect("BufferedReporter lock poisoned")
            .clear();
        self.stdout
            .lock()
            .expect("BufferedReporter lock poisoned")
            .clear();
        self.stderr
            .lock()
            .expect("BufferedReporter lock poisoned")
            .clear();
    }

    /// Total de mensajes acumulados de cualquier nivel
    pub fn total(&self) -> usize {
        self.warnings
            .lock()
            .expect("BufferedReporter lock poisoned")
            .len()
            + self
                .errors
                .lock()
                .expect("BufferedReporter lock poisoned")
                .len()
            + self
                .infos
                .lock()
                .expect("BufferedReporter lock poisoned")
                .len()
            + self
                .stdout
                .lock()
                .expect("BufferedReporter lock poisoned")
                .len()
            + self
                .stderr
                .lock()
                .expect("BufferedReporter lock poisoned")
                .len()
    }
}

impl Default for BufferedReporter {
    fn default() -> Self {
        Self::new()
    }
}

impl Reporter for BufferedReporter {
    fn warn(&self, message: &str) {
        self.warnings
            .lock()
            .expect("BufferedReporter lock poisoned")
            .push(message.to_owned());
    }
    fn error(&self, message: &str) {
        self.errors
            .lock()
            .expect("BufferedReporter lock poisoned")
            .push(message.to_owned());
    }
    fn info(&self, message: &str) {
        self.infos
            .lock()
            .expect("BufferedReporter lock poisoned")
            .push(message.to_owned());
    }

    fn raw_stdout(&self, message: &str) {
        self.stdout
            .lock()
            .expect("BufferedReporter lock poisoned")
            .push(message.to_owned());
    }

    fn raw_stderr(&self, message: &str) {
        self.stderr
            .lock()
            .expect("BufferedReporter lock poisoned")
            .push(message.to_owned());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reporter_levels_allow_expected_verbosity() {
        assert!(ReporterLevel::Info.allows_info());
        assert!(ReporterLevel::Info.allows_warn());

        assert!(!ReporterLevel::Warning.allows_info());
        assert!(ReporterLevel::Warning.allows_warn());

        assert!(!ReporterLevel::Error.allows_info());
        assert!(!ReporterLevel::Error.allows_warn());
    }

    #[test]
    fn stderr_reporter_default_is_error_level() {
        let reporter = StderrReporter::default();
        assert_eq!(reporter.level(), ReporterLevel::Error);
    }
}
