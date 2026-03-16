pub mod parser;
pub mod shell;

pub use shell::reporter::{
    Reporter, StderrReporter, SilentReporter, BufferedReporter,
};
pub use shell::guard::{Guard, GuardError, CompositeGuard, default_guard};
pub use shell::translator::subsystem::Subsystem;
pub use shell::translator::resolver::{
    Resolve, ResolvedCommand, ResolverError, SuggestionResolver,
};
pub use shell::translator::{TranslationPipeline, TranslationError};
pub use shell::executor::{Executor, ExecutionConfig, ExecutionResult};