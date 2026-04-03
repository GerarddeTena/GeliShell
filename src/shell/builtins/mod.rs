pub mod cd;
pub mod clear;
mod customization;
pub mod exit;
pub mod export;
pub mod gerisabet;
pub mod g_jump;
pub mod history;
pub mod source;
pub mod unset;

use crate::parser::ast::ASTNode;
use crate::shell::reporter::Reporter;
use crate::t;
use g_jump::{GJumpBuiltin, history::GHistory};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

// ══════════════════════════════════════════════════════════════
// BuiltinResult
// ══════════════════════════════════════════════════════════════

#[derive(Debug)]
pub enum BuiltinResult {
    Handled,
    NotABuiltin,
    Exit(i32),
}

// ══════════════════════════════════════════════════════════════
// trait Builtin
// ══════════════════════════════════════════════════════════════

pub trait Builtin: Send + Sync {
    fn name(&self) -> &'static str;
    fn execute(&self, args: &[String], reporter: &dyn Reporter) -> BuiltinResult;
}

// ══════════════════════════════════════════════════════════════
// BuiltinRegistry
// ══════════════════════════════════════════════════════════════

pub struct BuiltinRegistry {
    builtins: Vec<Box<dyn Builtin>>,
    history: Vec<String>,
    g_history: Arc<Mutex<GHistory>>,
}

impl BuiltinRegistry {
    pub fn new() -> Self {
        let g_history = Arc::new(Mutex::new(GHistory::load()));
        // Shared previous-directory state between CdBuiltin and GJumpBuiltin.
        // Avoids setting OLDPWD in the process environment from the REPL hot path.
        let oldpwd: Arc<Mutex<Option<PathBuf>>> = Arc::new(Mutex::new(None));

        Self {
            builtins: vec![
                Box::new(cd::CdBuiltin::new(Arc::clone(&oldpwd))),
                Box::new(clear::ClearBuiltin),
                Box::new(exit::ExitBuiltin),
                Box::new(export::ExportBuiltin),
                Box::new(gerisabet::GerisabetBuiltin),
                Box::new(source::SourceBuiltin),
                Box::new(unset::UnsetBuiltin),
                Box::new(GJumpBuiltin::new(Arc::clone(&g_history), oldpwd)),
            ],
            history: Vec::new(),
            g_history,
        }
    }

    /// Registra el cwd en el historial de g
    /// Llamar después de cada cd exitoso y después de cada comando
    pub fn record_g_visit(&self) {
        GJumpBuiltin::record_visit(&self.g_history);
    }

    pub fn push_history(&mut self, cmd: String) {
        self.history.push(cmd);
    }

    pub fn history(&self) -> &[String] {
        &self.history
    }

    pub fn g_completion_paths(&self, limit: usize) -> Vec<String> {
        match self.g_history.lock() {
            Ok(history) => history.completion_candidates(limit),
            Err(_) => Vec::new(),
        }
    }

    pub fn try_execute(&mut self, node: &ASTNode, reporter: &dyn Reporter) -> BuiltinResult {
        let ASTNode::Command(cmd) = node else {
            return BuiltinResult::NotABuiltin;
        };

        // history especial — necesita acceso al Vec interno
        if cmd.name == "history" {
            let args: Vec<String> = cmd
                .args
                .iter()
                .filter_map(|t| t.as_str().map(str::to_owned))
                .collect();
            if args.contains(&"--clear".to_owned()) {
                self.history.clear();
                reporter.info(&t!("builtin.history.cleared"));
            } else {
                for (i, entry) in self.history.iter().enumerate() {
                    reporter.info(&t!("builtin.history.entry", num = format!("{:4}", i + 1), entry = entry));
                }
            }
            return BuiltinResult::Handled;
        }

        let args: Vec<String> = cmd
            .args
            .iter()
            .filter_map(|t| t.as_str().map(str::to_owned))
            .collect();

        for builtin in &self.builtins {
            if builtin.name() == cmd.name {
                return builtin.execute(&args, reporter);
            }
        }

        BuiltinResult::NotABuiltin
    }
}
