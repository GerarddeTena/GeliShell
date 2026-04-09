pub mod context;
pub mod step;
pub mod steps;

use crate::parser::ast::ASTNode;
use crate::shell::reporter::Reporter;
use crate::shell::translator::commands_map::CommandMap;
use crate::shell::translator::pipeline::context::TranslationContext;
use crate::shell::translator::pipeline::step::{PipelineError, StepResult, TranslationStep};
use crate::shell::translator::pipeline::steps::{
    CommandResolver, FlagResolver, NodeDecomposer, SubsystemMapper, VariableExpander,
};
use crate::shell::translator::resolver::{Resolve, SuggestionResolver};
use crate::shell::translator::subsystem::Subsystem;
use crate::t;
use std::sync::Arc;

pub use context::{CommandFragment, FragmentOperator, StepSnapshot};
pub use step::PipelineError as TranslationError;

// ══════════════════════════════════════════════════════════════
// TranslationPipeline — orchestrador
// ══════════════════════════════════════════════════════════════

pub struct TranslationPipeline {
    steps: Vec<Box<dyn TranslationStep>>,
    map: Arc<CommandMap>,
    subsystem: Subsystem,
}

impl TranslationPipeline {
    /// Constructor con steps por defecto y SuggestionResolver estándar
    pub fn new(map: Arc<CommandMap>, subsystem: Subsystem) -> Self {
        let resolver: Arc<dyn Resolve> = Arc::new(SuggestionResolver::new());
        Self::with_resolver(map, subsystem, resolver)
    }

    /// Constructor con resolver personalizado — útil para tests
    pub fn with_resolver(
        map: Arc<CommandMap>,
        subsystem: Subsystem,
        resolver: Arc<dyn Resolve>,
    ) -> Self {
        let steps: Vec<Box<dyn TranslationStep>> = vec![
            Box::new(NodeDecomposer::new()),
            Box::new(CommandResolver::new()),
            Box::new(FlagResolver::new()),
            Box::new(VariableExpander::new()),
            Box::new(SubsystemMapper::new(Arc::clone(&resolver))),
        ];

        Self {
            steps,
            map,
            subsystem,
        }
    }

    /// Punto de entrada único — traduce un ASTNode a String nativo
    ///
    /// # Errors
    /// - `PipelineError::Fatal`    — el pipeline no puede continuar
    /// - `PipelineError::Degraded` — resultado parcial disponible
    pub fn run(&self, node: &ASTNode, reporter: &dyn Reporter) -> Result<String, PipelineError> {
        self.run_resolving(node, reporter).map(|(s, _)| s)
    }

    /// Same as `run()` but also returns the `ResolvedCommand` of the first fragment.
    /// Used by the REPL handler to drive `ModalSelector` when `SelectorMode` requires it.
    pub fn run_resolving(
        &self,
        node: &ASTNode,
        reporter: &dyn Reporter,
    ) -> Result<
        (
            String,
            Option<crate::shell::translator::resolver::ResolvedCommand>,
        ),
        PipelineError,
    > {
        let mut ctx = TranslationContext::new(node, &self.subsystem, &self.map);

        for step in &self.steps {
            match step.process(&mut ctx, reporter)? {
                StepResult::Continue => continue,
                StepResult::Done(output) => {
                    ctx.output = Some(output.clone());
                    ctx.snapshot(step.name());
                    let resolved = ctx.fragments.into_iter().next().and_then(|f| f.resolved);
                    return Ok((output, resolved));
                }
            }
        }

        let resolved = ctx.fragments.first().and_then(|f| f.resolved.clone());

        let output = if let Some(out) = ctx.output.take() {
            out
        } else {
            ctx.assemble()
        };

        reporter.info(&t!("pipeline.complete", output = output));

        Ok((output, resolved))
    }

    /// Devuelve los snapshots del último run — solo en debug builds
    /// En release siempre devuelve Vec vacío
    pub fn run_with_trace(
        &self,
        node: &ASTNode,
        reporter: &dyn Reporter,
    ) -> Result<(String, Vec<StepSnapshot>), PipelineError> {
        let mut ctx = TranslationContext::new(node, &self.subsystem, &self.map);

        for step in &self.steps {
            match step.process(&mut ctx, reporter)? {
                StepResult::Continue => continue,
                StepResult::Done(output) => {
                    ctx.output = Some(output.clone());
                    ctx.snapshot(step.name());
                    return Ok((output, ctx.snapshots));
                }
            }
        }

        let output = if let Some(out) = ctx.output.take() {
            out
        } else {
            ctx.assemble()
        };

        Ok((output, ctx.snapshots))
    }
}

// ══════════════════════════════════════════════════════════════
// Tests de integración del pipeline completo
// ══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ast::{ASTNode, Command};
    use crate::parser::token::Token;
    use crate::shell::reporter::{BufferedReporter, SilentReporter};
    use crate::shell::translator::commands_map::{load, load_from_str};

    fn make_command_node(name: &str, args: Vec<&str>) -> ASTNode {
        ASTNode::Command(Command {
            name: name.to_owned(),
            args: args.iter().map(|a| Token::Word(a.to_string())).collect(),
            redirections: vec![],
        })
    }

    fn make_pipeline() -> Arc<CommandMap> {
        let result = load().expect("commands.toml must be valid in tests");
        Arc::new(result.map)
    }

    fn make_pipeline_from_toml(raw_toml: &str) -> Arc<CommandMap> {
        let result = load_from_str(raw_toml).expect("inline command map fixture must be valid");
        Arc::new(result.map)
    }

    // ──────────────────────────────────────────────────────────
    // Tests de comando simple
    // ──────────────────────────────────────────────────────────

    #[test]
    fn translates_list_to_bash() {
        let map = make_pipeline();
        let pipeline = TranslationPipeline::new(map, Subsystem::Bash);
        let reporter = SilentReporter::new();
        let node = make_command_node("list", vec![]);
        let result = pipeline.run(&node, &reporter).unwrap();
        assert_eq!(result, "ls");
    }

    #[test]
    fn translates_list_to_powershell() {
        let map = make_pipeline();
        let pipeline = TranslationPipeline::new(map, Subsystem::PowerShell);
        let reporter = SilentReporter::new();
        let node = make_command_node("list", vec![]);
        let result = pipeline.run(&node, &reporter).unwrap();
        assert_eq!(result, "Get-ChildItem");
    }

    #[test]
    fn passthrough_for_unknown_command() {
        let map = make_pipeline();
        let pipeline = TranslationPipeline::new(map, Subsystem::Bash);
        let reporter = SilentReporter::new();
        let node = make_command_node("git", vec!["status"]);
        let result = pipeline.run(&node, &reporter).unwrap();
        assert_eq!(result, "git status");
    }

    // ──────────────────────────────────────────────────────────
    // Tests de nodos compuestos
    // ──────────────────────────────────────────────────────────

    #[test]
    fn translates_and_node() {
        let map = make_pipeline();
        let pipeline = TranslationPipeline::new(Arc::clone(&map), Subsystem::Bash);
        let reporter = SilentReporter::new();
        let node = ASTNode::And(
            Box::new(make_command_node("list", vec![])),
            Box::new(make_command_node("clear", vec![])),
        );
        let result = pipeline.run(&node, &reporter).unwrap();
        assert!(result.contains("&&"), "expected && in '{result}'");
    }

    #[test]
    fn cmd_and_uses_single_ampersand() {
        let map = make_pipeline();
        let pipeline = TranslationPipeline::new(Arc::clone(&map), Subsystem::Cmd);
        let reporter = SilentReporter::new();
        let node = ASTNode::And(
            Box::new(make_command_node("list", vec![])),
            Box::new(make_command_node("clear", vec![])),
        );
        let result = pipeline.run(&node, &reporter).unwrap();
        assert!(result.contains(" & "), "expected ' & ' in '{result}'");
        assert!(!result.contains("&&"), "cmd should not use &&");
    }

    #[test]
    fn translates_pipeline_node() {
        let map = make_pipeline();
        let pipeline = TranslationPipeline::new(Arc::clone(&map), Subsystem::Bash);
        let reporter = SilentReporter::new();
        let node = ASTNode::Pipeline(vec![
            make_command_node("list", vec![]),
            make_command_node("search", vec!["foo"]),
        ]);
        let result = pipeline.run(&node, &reporter).unwrap();
        assert!(result.contains(" | "), "expected pipe in '{result}'");
    }

    // ──────────────────────────────────────────────────────────
    // Tests de trazabilidad
    // ──────────────────────────────────────────────────────────

    #[test]
    fn run_with_trace_captures_snapshots_in_debug() {
        let map = make_pipeline();
        let pipeline = TranslationPipeline::new(map, Subsystem::Bash);
        let reporter = SilentReporter::new();
        let node = make_command_node("list", vec![]);
        let (output, snapshots) = pipeline.run_with_trace(&node, &reporter).unwrap();

        assert_eq!(output, "ls");
        // En debug build debe haber al menos un snapshot por step
        #[cfg(debug_assertions)]
        assert!(!snapshots.is_empty(), "expected snapshots in debug build");
    }

    #[test]
    fn buffered_reporter_captures_all_step_messages() {
        let map = make_pipeline();
        let pipeline = TranslationPipeline::new(map, Subsystem::Bash);
        let reporter = BufferedReporter::new();
        let node = make_command_node("list", vec![]);
        pipeline.run(&node, &reporter).unwrap();

        // Debe haber mensajes info de cada step
        assert!(
            reporter.infos().len() >= 3,
            "expected info messages from steps, got: {:?}",
            reporter.infos()
        );
    }

    #[test]
    fn translates_inline_custom_command_and_reports_canonical_match() {
        let raw_toml = r#"
[[commands]]
name = "gerisabet"
description = "assistant command"
category = "dev"
translate = { bash = { exact = "echo", suggestions = [] }, zsh = { exact = "echo", suggestions = [] }, fish = { exact = "echo", suggestions = [] }, powershell = { exact = "Write-Output", suggestions = ["echo"] }, cmd = { exact = "echo", suggestions = [] } }
"#;
        let map = make_pipeline_from_toml(raw_toml);
        let pipeline = TranslationPipeline::new(map, Subsystem::PowerShell);
        let reporter = BufferedReporter::new();
        let node = make_command_node("gerisabet", vec!["hola"]);

        let result = pipeline.run(&node, &reporter).unwrap();
        assert_eq!(result, "Write-Output hola");
        assert!(
            reporter
                .infos()
                .iter()
                .any(|msg| msg.contains("gerisabet") && msg.contains("canonical match found")),
            "expected canonical match trace in infos, got: {:?}",
            reporter.infos()
        );
    }

    // ──────────────────────────────────────────────────────────
    // Tests de reverse lookup bidireccional (Problema 1)
    // ──────────────────────────────────────────────────────────

    /// Usuario en subsistema PowerShell escribe `ls` (nativo de Bash).
    /// El pipeline debe detectarlo vía reverse lookup y traducirlo a `Get-ChildItem`.
    #[test]
    fn reverse_lookup_translates_ls_to_powershell() {
        let map = make_pipeline();
        let pipeline = TranslationPipeline::new(map, Subsystem::PowerShell);
        let reporter = SilentReporter::new();
        let node = make_command_node("ls", vec![]);
        let result = pipeline.run(&node, &reporter).unwrap();
        assert_eq!(result, "Get-ChildItem");
    }

    /// Usuario en subsistema Bash escribe `Set-Location` (nativo de PowerShell).
    /// `cd` es el exact de bash para change-dir, `Set-Location` es único → reverse lookup válido.
    #[test]
    fn reverse_lookup_translates_set_location_to_bash() {
        let map = make_pipeline();
        let pipeline = TranslationPipeline::new(map, Subsystem::Bash);
        let reporter = SilentReporter::new();
        let node = make_command_node("Set-Location", vec!["/tmp"]);
        let result = pipeline.run(&node, &reporter).unwrap();
        assert_eq!(result, "cd /tmp");
    }

    /// El reverse lookup traza el paso de normalización en el reporter.
    #[test]
    fn reverse_lookup_logs_reverse_lookup_trace() {
        let map = make_pipeline();
        let pipeline = TranslationPipeline::new(map, Subsystem::PowerShell);
        let reporter = BufferedReporter::new();
        let node = make_command_node("ls", vec![]);
        pipeline.run(&node, &reporter).unwrap();
        assert!(
            reporter
                .infos()
                .iter()
                .any(|msg| msg.contains("reverse lookup")),
            "expected 'reverse lookup' trace, got: {:?}",
            reporter.infos()
        );
    }

    /// Comando nativo sin equivalente canónico → pass-through intacto.
    #[test]
    fn reverse_lookup_passthrough_for_truly_unknown_native() {
        let map = make_pipeline();
        let pipeline = TranslationPipeline::new(map, Subsystem::Bash);
        let reporter = SilentReporter::new();
        let node = make_command_node("docker", vec!["ps"]);
        let result = pipeline.run(&node, &reporter).unwrap();
        assert_eq!(result, "docker ps");
    }
}
