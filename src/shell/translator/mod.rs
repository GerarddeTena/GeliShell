// src/shell/translator/mod.rs

pub mod commands_map;
pub mod pipeline;
pub mod resolver;
pub mod subsystem;

pub use commands_map::{
    CommandDef, CommandMap, CommandMapError, FlagDef, LoadResult, SubsystemTranslations,
    TranslationEntry, ValidationWarning, load,
};
pub use pipeline::{TranslationError, TranslationPipeline};
pub use resolver::{Resolve, ResolvedCommand, ResolverError, SuggestionResolver};
pub use subsystem::{ScoredSuggestion, Subsystem, SuggestionKind};
