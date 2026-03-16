pub mod cd;
pub mod clear;
pub mod exit;
pub mod export;
pub mod history;
pub mod source;
pub mod unset;

use crate::parser::ast::ASTNode;
use crate::shell::reporter::Reporter;

// ══════════════════════════════════════════════════════════════
// BuiltinResult
// ══════════════════════════════════════════════════════════════

#[derive(Debug)]
pub enum BuiltinResult {
    /// El builtin manejó el comando — no pasar al executor
    Handled,
    /// El comando no es un builtin — continúa con el flujo normal
    NotABuiltin,
    /// El builtin pide cerrar la shell
    Exit(i32),
}

// ══════════════════════════════════════════════════════════════
// trait Builtin
// ══════════════════════════════════════════════════════════════

pub trait Builtin: Send + Sync {
    fn name(&self) -> &'static str;
    fn execute(
        &self,
        args:     &[String],
        reporter: &dyn Reporter,
    ) -> BuiltinResult;
}

// ══════════════════════════════════════════════════════════════
// BuiltinRegistry
// ══════════════════════════════════════════════════════════════

pub struct BuiltinRegistry {
    builtins: Vec<Box<dyn Builtin>>,
    history:  Vec<String>,
}

impl BuiltinRegistry {
    pub fn new() -> Self {
        Self {
            builtins: vec![
                Box::new(cd::CdBuiltin),
                Box::new(clear::ClearBuiltin),
                Box::new(exit::ExitBuiltin),
                Box::new(export::ExportBuiltin),
                Box::new(history::HistoryBuiltin),
                Box::new(source::SourceBuiltin),
                Box::new(unset::UnsetBuiltin),
            ],
            history: Vec::new(),
        }
    }

    /// Añade un comando al historial interno
    pub fn push_history(&mut self, cmd: String) {
        self.history.push(cmd);
    }

    /// Devuelve el historial completo
    pub fn history(&self) -> &[String] {
        &self.history
    }

    /// Comprueba si el ASTNode es un builtin y lo ejecuta
    pub fn try_execute(
        &mut self,
        node:     &ASTNode,
        reporter: &dyn Reporter,
    ) -> BuiltinResult {
        let ASTNode::Command(cmd) = node else {
            return BuiltinResult::NotABuiltin;
        };

        // Caso especial: history necesita acceso al historial interno
        if cmd.name == "history" {
            let args: Vec<String> = cmd.args.iter()
                .filter_map(|t| t.as_str().map(str::to_owned))
                .collect();
            if args.contains(&"--clear".to_owned()) {
                self.history.clear();
                reporter.info("history cleared");
            } else {
                for (i, entry) in self.history.iter().enumerate() {
                    println!("{:4}  {}", i + 1, entry);
                }
            }
            return BuiltinResult::Handled;
        }

        // Resto de builtins
        let args: Vec<String> = cmd.args.iter()
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