// src/shell/translator/mod.rs

pub mod commands_map;
pub mod subsystem;
pub mod resolver;
pub mod pipeline;

pub use commands_map::{
    CommandDef, CommandMap, CommandMapError,
    FlagDef, LoadResult, SubsystemTranslations,
    TranslationEntry, ValidationWarning, load,
};
pub use subsystem::{ScoredSuggestion, SuggestionKind, Subsystem};
pub use resolver::{Resolve, ResolvedCommand, ResolverError, SuggestionResolver};
pub use pipeline::{TranslationPipeline, TranslationError};
