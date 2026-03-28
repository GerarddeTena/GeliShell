pub mod parser;
pub mod shell;

pub use shell::config::bootstrap;
pub use shell::executor::{ExecutionConfig, ExecutionResult, Executor};
pub use shell::guard::{
    CompositeGuard, Guard, GuardError, NormalizedCompositeGuard, default_guard,
    default_guard_normalized,
};
pub use shell::reporter::{BufferedReporter, Reporter, SilentReporter, StderrReporter};
pub use shell::translator::resolver::{
    Resolve, ResolvedCommand, ResolverError, SuggestionResolver,
};
pub use shell::translator::subsystem::Subsystem;
pub use shell::translator::{TranslationError, TranslationPipeline};
