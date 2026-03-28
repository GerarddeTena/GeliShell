pub mod frequency;
pub mod history;
pub mod matcher;

use crate::shell::builtins::{Builtin, BuiltinResult};
use crate::shell::reporter::Reporter;
use history::GHistory;
use std::sync::{Arc, Mutex};

// ══════════════════════════════════════════════════════════════
// GJumpBuiltin — el navegador inteligente de GeliShell
// ══════════════════════════════════════════════════════════════

pub struct GJumpBuiltin {
    history: Arc<Mutex<GHistory>>,
}

impl GJumpBuiltin {
    pub fn new(history: Arc<Mutex<GHistory>>) -> Self {
        Self { history }
    }

    /// Registra el cwd en el historial — llamar después de cada cd
    pub fn record_visit(history: &Arc<Mutex<GHistory>>) {
        if let Ok(cwd) = std::env::current_dir() {
            let path = cwd.to_string_lossy().replace('\\', "/");
            if let Ok(mut h) = history.lock() {
                h.record_visit(&path);
            }
        }
    }

    fn jump_to(&self, target: &str, reporter: &dyn Reporter) {
        if let Ok(current) = std::env::current_dir() {
            // SAFETY: the REPL loop is single-threaded at this point;
            // no concurrent thread reads OLDPWD while we write it.
            unsafe {
                std::env::set_var("OLDPWD", current.to_string_lossy().as_ref());
            }
        }

        match std::env::set_current_dir(target) {
            Ok(_) => {
                // SAFETY: same single-threaded REPL guarantee as OLDPWD above.
                unsafe {
                    std::env::set_var("PWD", target);
                }
                reporter.info(&format!("g: → {target}"));
            }
            Err(e) => reporter.error(&format!("g: {e}")),
        }
    }

    fn show_history(&self, reporter: &dyn Reporter) {
        let Ok(history) = self.history.lock() else {
            reporter.error("g: could not read history");
            return;
        };

        if history.is_empty() {
            reporter.info(
                "g: no history yet — \
                 navigate directories and GeliShell will learn them",
            );
            return;
        }

        let top = history.top(10);

        println!();
        println!(
            "  {:<4} {:<8} {:<12} {}",
            "#", "visits", "last seen", "path"
        );
        println!("  {}", "─".repeat(60));

        for (i, (entry, score)) in top.iter().enumerate() {
            let last = frequency::elapsed_display(entry.last_visit);

            // Acorta el home a ~
            let display_path = shorten_home(&entry.path);

            println!(
                "  {:<4} {:<8} {:<12} {}  \x1b[38;5;240m({:.0})\x1b[0m",
                i + 1,
                entry.visits,
                last,
                display_path,
                score,
            );
        }
        println!();
    }
}

impl Builtin for GJumpBuiltin {
    fn name(&self) -> &'static str {
        "g"
    }

    fn execute(&self, args: &[String], reporter: &dyn Reporter) -> BuiltinResult {
        match args.first().map(String::as_str) {
            // g — sin args: muestra el top 10
            None => {
                self.show_history(reporter);
            }

            // g --clear — limpia el historial
            Some("--clear") => {
                if let Ok(mut h) = self.history.lock() {
                    h.clear();
                    reporter.info("g: history cleared");
                }
            }

            // g - — vuelve al directorio anterior
            Some("-") => match std::env::var("OLDPWD") {
                Ok(prev) => self.jump_to(&prev, reporter),
                Err(_) => reporter.warn("g: no previous directory"),
            },

            // g <pattern> — salta al mejor match
            Some(pattern) => {
                let best = self
                    .history
                    .lock()
                    .ok()
                    .and_then(|h| h.best_match(pattern).map(|e| e.path.clone()));

                match best {
                    Some(target) => {
                        self.jump_to(&target, reporter);
                        // Registra la visita al llegar
                        GJumpBuiltin::record_visit(&self.history);
                    }
                    None => {
                        reporter.warn(&format!(
                            "g: no match for '{pattern}' — \
                             visit the directory first so GeliShell learns it"
                        ));
                    }
                }
            }
        }

        BuiltinResult::Handled
    }
}

// ══════════════════════════════════════════════════════════════
// Helper
// ══════════════════════════════════════════════════════════════

fn shorten_home(path: &str) -> String {
    let home_var = if cfg!(target_os = "windows") {
        std::env::var("USERPROFILE")
    } else {
        std::env::var("HOME")
    };

    if let Ok(home) = home_var {
        let home_normalized = home.replace('\\', "/");
        if path.starts_with(&home_normalized) {
            let stripped = &path[home_normalized.len()..];
            return if stripped.is_empty() {
                "~".to_owned()
            } else {
                format!("~{stripped}")
            };
        }
    }
    path.to_owned()
}
