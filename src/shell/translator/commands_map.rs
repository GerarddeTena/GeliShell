use serde::Deserialize;
use std::collections::HashMap;
use thiserror::Error;

static RAW_COMMANDS: &str = include_str!("../../commands/commands.toml");

#[derive(Debug, Error)]
pub enum CommandMapError {
    #[error("failed to deserialize commands.toml: {0}")]
    Deserialize(#[from] toml::de::Error),

    #[error("commands.toml is empty — no commands loaded")]
    Empty,
}

// ------- Validación Acumulativa ---------
#[derive(Debug, Deserialize)]
pub enum ValidationWarning {
    EmptyExact { command: String, subsystem: String },
    EmptySuggestions { command: String, subsystem: String },
    MissingSubsystem { command: String, subsystem: String },
    // For TOML doubleEntries:
    DuplicateCommand { name: String },
}

impl std::fmt::Display for ValidationWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyExact { command, subsystem } => {
                write!(f, "warning: '{command}' has empty exact for '{subsystem}'")
            }
            Self::EmptySuggestions { command, subsystem } => {
                write!(
                    f,
                    "warning: '{command}' has no suggestions for '{subsystem}'"
                )
            }
            Self::MissingSubsystem { command, subsystem } => {
                write!(
                    f,
                    "warning: '{command}' missing translation for '{subsystem}'"
                )
            }
            Self::DuplicateCommand { name } => {
                write!(f, "warning: duplicate command '{name}' — last entry wins")
            }
        }
    }
}

// ------- Resultado de carga --------
pub struct LoadResult {
    pub map: CommandMap,
    pub warnings: Vec<ValidationWarning>,
}

impl LoadResult {
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    pub fn report(&self, reporter: &dyn crate::shell::reporter::Reporter) {
        for w in &self.warnings {
            reporter.warn(&w.to_string());
        }
    }
}

// ------- Estructuras de deserializacion

#[derive(Debug, Deserialize)]
pub struct CommandsFile {
    pub commands: Vec<CommandDef>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CommandDef {
    pub name: String,
    pub description: String,
    pub category: String,
    pub translate: SubsystemTranslations,
    #[serde(default)]
    pub flags: Vec<FlagDef>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TranslationEntry {
    pub exact: String,
    #[serde(default)]
    pub suggestions: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[derive(Default)]
pub struct SubsystemTranslations {
    pub bash: Option<TranslationEntry>,
    pub zsh: Option<TranslationEntry>,
    pub fish: Option<TranslationEntry>,
    pub powershell: Option<TranslationEntry>,
    pub cmd: Option<TranslationEntry>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct FlagDef {
    pub canonical: String,
    pub bash: Option<String>,
    pub zsh: Option<String>,
    pub fish: Option<String>,
    pub powershell: Option<String>,
    pub cmd: Option<String>,
}
// ══════════════════════════════════════════════════════════════
// Métodos de acceso en SubsystemTranslations y FlagDef
// Centralizan el lookup por subsistema — un solo punto de cambio
// ══════════════════════════════════════════════════════════════

const SUBSYSTEMS: &[&str] = &["bash", "zsh", "fish", "powershell", "cmd"];

impl SubsystemTranslations {
    // Devuelve la TranslationEntry para un subsistema dado por nombre
    pub(crate) fn get_by_name(&self, subsystem: &str) -> Option<&TranslationEntry> {
        match subsystem {
            "bash" => self.bash.as_ref(),
            "zsh" => self.zsh.as_ref(),
            "fish" => self.fish.as_ref(),
            "powershell" => self.powershell.as_ref(),
            "cmd" => self.cmd.as_ref(),
            _ => None,
        }
    }

    // Itera todos los subsistemas con su nombre — para validación
    pub(crate) fn iter_named(&self) -> impl Iterator<Item = (&str, Option<&TranslationEntry>)> {
        SUBSYSTEMS
            .iter()
            .map(move |&name| (name, self.get_by_name(name)))
    }
}

impl FlagDef {
    pub(crate) fn get_by_name(&self, subsystem: &str) -> Option<&str> {
        match subsystem {
            "bash" => self.bash.as_deref(),
            "zsh" => self.zsh.as_deref(),
            "fish" => self.fish.as_deref(),
            "powershell" => self.powershell.as_deref(),
            "cmd" => self.cmd.as_deref(),
            _ => None,
        }
    }
}

impl TranslationEntry {
    pub(crate) fn is_valid(&self) -> bool {
        !self.exact.trim().is_empty() || !self.suggestions.is_empty()
    }
}
pub struct CommandMap {
    index: HashMap<String, CommandDef>,
}

impl CommandMap {
    pub fn get(&self, name: &str) -> Option<&CommandDef> {
        self.index.get(name)
    }

    pub fn by_category(&self, category: &str) -> Vec<&CommandDef> {
        self.index
            .values()
            .filter(|cmd| cmd.category == category)
            .collect()
    }

    pub fn all_categories(&self) -> Vec<&str> {
        let mut cats: Vec<&str> = self
            .index
            .values()
            .map(|cmd| cmd.category.as_str())
            .collect();
        cats.sort_unstable();
        cats.dedup();
        cats
    }

    pub fn len(&self) -> usize {
        self.index.len()
    }

    pub fn is_empty(&self) -> bool {
        self.index.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &CommandDef> {
        self.index.values()
    }
}

pub fn load() -> Result<LoadResult, CommandMapError> {
    let file: CommandsFile = toml::from_str(&RAW_COMMANDS)?;

    if file.commands.is_empty() {
        return Err(CommandMapError::Empty);
    }

    let mut warnings = Vec::new();
    let mut index = HashMap::with_capacity(file.commands.len());

    for cmd in file.commands {
        if index.contains_key(&cmd.name) {
            warnings.push(ValidationWarning::DuplicateCommand {
                name: cmd.name.clone(),
            });
        }

        for (subsystem, entry_opt) in cmd.translate.iter_named() {
            match entry_opt {
                None => {
                    warnings.push(ValidationWarning::MissingSubsystem {
                        command: cmd.name.clone(),
                        subsystem: subsystem.to_owned(),
                    });
                }
                Some(entry) => {
                    if !entry.is_valid() {
                        warnings.push(ValidationWarning::EmptyExact {
                            command: cmd.name.clone(),
                            subsystem: subsystem.to_owned(),
                        });
                    } else if entry.suggestions.is_empty() {
                        warnings.push(ValidationWarning::EmptySuggestions {
                            command: cmd.name.clone(),
                            subsystem: subsystem.to_owned(),
                        });
                    }
                }
            }
        }
        index.insert(cmd.name.clone(), cmd);
    }

    Ok(LoadResult {
        map: CommandMap { index },
        warnings,
    })
}

#[cfg(test)]
mod tests {
    use crate::shell::translator::FlagDef;

    #[test]
    fn flag_get_by_name_returns_correct_translation() {
        let flag = FlagDef {
            canonical:  "--recursive".to_owned(),
            bash:       Some("-r".to_owned()),
            zsh:        Some("-r".to_owned()),
            fish:       Some("-r".to_owned()),
            powershell: Some("-Recurse".to_owned()),
            cmd:        Some("/s".to_owned()),
        };
        assert_eq!(flag.get_by_name("bash"),       Some("-r"));
        assert_eq!(flag.get_by_name("powershell"),  Some("-Recurse"));
        assert_eq!(flag.get_by_name("cmd"),         Some("/s"));
        assert_eq!(flag.get_by_name("unknown"),     None);
    }
}