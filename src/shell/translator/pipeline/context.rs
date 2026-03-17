use crate::parser::ast::ASTNode;
use crate::shell::translator::commands_map::{CommandDef, CommandMap};
use crate::shell::translator::resolver::ResolvedCommand;
use crate::shell::translator::subsystem::Subsystem;

// ══════════════════════════════════════════════════════════════
// FragmentOperator — operador que une este fragment al siguiente
// ══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, PartialEq)]
pub enum FragmentOperator {
    And,      // &&
    Or,       // ||
    Sequence, // ;
    Pipe,     // |
}

impl FragmentOperator {
    /// Traduce el operador a la sintaxis nativa del subsistema
    pub fn to_native(&self, subsystem: &Subsystem) -> &'static str {
        match self {
            Self::And => subsystem.and_operator(),
            Self::Or => subsystem.or_operator(),
            Self::Sequence => subsystem.sequence_operator(),
            Self::Pipe => " | ",
        }
    }
}

// ══════════════════════════════════════════════════════════════
// CommandFragment — unidad de trabajo entre steps
// ══════════════════════════════════════════════════════════════

/// Representa un comando simple ya descompuesto del ASTNode.
/// Viaja por el pipeline siendo enriquecido por cada step.
#[derive(Debug, Clone)]
pub struct CommandFragment {
    /// Nombre del comando — puede ser canónico o nativo
    pub command: String,

    /// Args resueltos — strings ya expandidos
    pub args: Vec<String>,

    /// Operador que conecta este fragment con el siguiente
    /// None si es el último fragment
    pub operator: Option<FragmentOperator>,

    /// true si debe ejecutarse en background
    pub background: bool,

    /// Llenado por CommandResolver — None si es comando pass-through
    pub command_def: Option<CommandDef>,

    /// Llenado por SubsystemMapper — None hasta ese step
    pub resolved: Option<ResolvedCommand>,
}

impl CommandFragment {
    pub fn new(command: String, args: Vec<String>) -> Self {
        Self {
            command,
            args,
            operator: None,
            background: false,
            command_def: None,
            resolved: None,
        }
    }

    /// String ejecutable final — usa resolved.preferred si está disponible
    pub fn to_native_string(&self, _subsystem: &Subsystem) -> String {
        let base = self
            .resolved
            .as_ref()
            .map(|r| r.preferred.as_str())
            .unwrap_or(&self.command);

        if self.args.is_empty() {
            base.to_owned()
        } else {
            format!("{base} {}", self.args.join(" "))
        }
    }
}

// ══════════════════════════════════════════════════════════════
// StepSnapshot — foto del estado en un step concreto
// Solo se construye en debug — vacío en release
// ══════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct StepSnapshot {
    pub step_name: &'static str,
    pub fragments: Vec<CommandFragment>,
    pub output: Option<String>,
}

// ══════════════════════════════════════════════════════════════
// TranslationContext — viaja por todos los steps
// ══════════════════════════════════════════════════════════════

pub struct TranslationContext<'a> {
    // ── Input inmutable — compartido por todos los steps ──────
    pub node: &'a ASTNode,
    pub subsystem: &'a Subsystem,
    pub map: &'a CommandMap,

    // ── Estado mutable — acumulado por los steps ──────────────
    /// Fragments descompuestos por NodeDecomposer
    /// Cada step posterior los enriquece in-place
    pub fragments: Vec<CommandFragment>,

    /// Output final — llenado por SubsystemMapper o Done
    pub output: Option<String>,

    // ── Snapshots — solo en debug builds ──────────────────────
    /// Historial de estados para trazabilidad en preprod
    /// En release build se compila a Vec vacío
    pub snapshots: Vec<StepSnapshot>,
}

impl<'a> TranslationContext<'a> {
    pub fn new(node: &'a ASTNode, subsystem: &'a Subsystem, map: &'a CommandMap) -> Self {
        Self {
            node,
            subsystem,
            map,
            fragments: Vec::new(),
            output: None,
            snapshots: Vec::new(),
        }
    }

    /// Toma un snapshot del estado actual — solo activo en debug
    /// En release compila a no-op (opt-cold-unlikely)
    pub fn snapshot(&mut self, step_name: &'static str) {
        #[cfg(debug_assertions)]
        self.snapshots.push(StepSnapshot {
            step_name,
            fragments: self.fragments.clone(),
            output: self.output.clone(),
        });

        // En release: no-op — cero overhead
        #[cfg(not(debug_assertions))]
        let _ = step_name;
    }

    /// true si todos los fragments tienen resolved llenado
    pub fn is_fully_resolved(&self) -> bool {
        self.fragments.iter().all(|f| f.resolved.is_some())
    }

    /// Ensambla el output final uniendo fragments con sus operadores
    pub fn assemble(&self) -> String {
        let mut parts: Vec<String> = Vec::with_capacity(self.fragments.len());

        for (i, fragment) in self.fragments.iter().enumerate() {
            let native = fragment.to_native_string(self.subsystem);

            let with_bg = if fragment.background {
                self.subsystem.background_wrap(&native)
            } else {
                native
            };

            parts.push(with_bg);

            // Añade el operador si no es el último fragment
            if i < self.fragments.len() - 1 {
                if let Some(op) = &fragment.operator {
                    parts.push(op.to_native(self.subsystem).to_owned());
                }
            }
        }

        parts.join("")
    }
}
