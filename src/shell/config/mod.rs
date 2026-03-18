pub mod bootstrap;
pub mod first_run;
pub mod history_store;

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;

// ══════════════════════════════════════════════════════════════
// Errores
// ══════════════════════════════════════════════════════════════

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("config file not found — first run required")]
    NotFound,

    #[error("failed to read config: {0}")]
    Read(#[from] std::io::Error),

    #[error("failed to parse config: {0}")]
    Parse(#[from] toml::de::Error),

    #[error("failed to serialize config: {0}")]
    Serialize(#[from] toml::ser::Error),
}

// ══════════════════════════════════════════════════════════════
// SelectorMode
// ══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SelectorMode {
    Always,
    Auto,
    Once,
}

impl Default for SelectorMode {
    fn default() -> Self {
        Self::Always
    }
}

impl std::fmt::Display for SelectorMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Always => write!(f, "always"),
            Self::Auto => write!(f, "auto"),
            Self::Once => write!(f, "once"),
        }
    }
}

// ══════════════════════════════════════════════════════════════
// ShellConfig
// ══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ShellConfig {
    pub behavior: BehaviorConfig,
    pub subsystem: SubsystemConfig,
    pub execution: ExecutionConfig,
    pub visual: VisualConfig,
    pub customization: CustomizationConfig,
    pub assistant: AssistantConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct BehaviorConfig {
    pub selector_mode: SelectorMode,
}

impl Default for BehaviorConfig {
    fn default() -> Self {
        Self {
            selector_mode: SelectorMode::Always,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SubsystemConfig {
    pub override_subsystem: String,
}

impl Default for SubsystemConfig {
    fn default() -> Self {
        Self {
            override_subsystem: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ExecutionConfig {
    pub capture_output: bool,
    pub capture_duration: bool,
    pub capture_command_trace: bool,
    pub timeout_secs: u64,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            capture_output: false,
            capture_duration: false,
            capture_command_trace: false,
            timeout_secs: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct VisualConfig {
    pub terminal_foreground_ansi256: u8,
    pub terminal_background_ansi256: u8,
    pub prompt_path_ansi256: u8,
    pub prompt_subsystem_ansi256: u8,
    pub prompt_name_ansi256: u8,
    pub prompt_dim_ansi256: u8,
    pub font_family: String,
}

impl Default for VisualConfig {
    fn default() -> Self {
        Self {
            terminal_foreground_ansi256: 253,
            terminal_background_ansi256: 0,
            prompt_path_ansi256: 253,
            prompt_subsystem_ansi256: 141,
            prompt_name_ansi256: 213,
            prompt_dim_ansi256: 240,
            font_family: "Cascadia Mono".to_owned(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct CustomizationConfig {
    pub custom_commands: Vec<CustomCommand>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CustomCommand {
    pub name: String,
    pub template: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AssistantModelVariant {
    Qwen05b,
    Qwen15b,
}

impl Default for AssistantModelVariant {
    fn default() -> Self {
        Self::Qwen05b
    }
}

impl AssistantModelVariant {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Qwen05b => "qwen-0.5b",
            Self::Qwen15b => "qwen-1.5b",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AssistantConfig {
    pub model_variant: AssistantModelVariant,
    pub rag_top_k: usize,
    pub auto_unload_after_secs: u64,
}

impl Default for AssistantConfig {
    fn default() -> Self {
        Self {
            model_variant: AssistantModelVariant::default(),
            rag_top_k: 4,
            auto_unload_after_secs: 300,
        }
    }
}

impl Default for ShellConfig {
    fn default() -> Self {
        Self {
            behavior: BehaviorConfig::default(),
            subsystem: SubsystemConfig::default(),
            execution: ExecutionConfig::default(),
            visual: VisualConfig::default(),
            customization: CustomizationConfig::default(),
            assistant: AssistantConfig::default(),
        }
    }
}

// ══════════════════════════════════════════════════════════════
// Persistencia
// ══════════════════════════════════════════════════════════════

impl ShellConfig {
    fn platform_config_root() -> PathBuf {
        if cfg!(target_os = "windows") {
            if let Ok(user_profile) = std::env::var("USERPROFILE") {
                return PathBuf::from(user_profile).join(".config");
            }
        } else if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home).join(".config");
        }

        dirs::config_dir().unwrap_or_else(|| PathBuf::from("."))
    }

    pub fn geli_config_dir() -> PathBuf {
        Self::platform_config_root().join("geliShell")
    }

    pub fn config_path() -> PathBuf {
        Self::geli_config_dir().join("config.toml")
    }

    pub fn command_history_path() -> PathBuf {
        Self::geli_config_dir().join("history.txt")
    }

    pub fn assistant_models_dir() -> PathBuf {
        Self::geli_config_dir().join("models")
    }

    pub fn docs_db_path(base: &Path) -> PathBuf {
        base.join("docs").join("docs.db")
    }

    pub fn assistant_docs_db_path() -> PathBuf {
        std::env::var("GELI_DOCS_DB_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| Self::docs_db_path(&Self::geli_config_dir()))
    }

    pub fn assistant_docs_dir() -> PathBuf {
        Self::geli_config_dir().join("docs")
    }

    pub async fn load_async() -> Result<Self, ConfigError> {
        let path = Self::config_path();
        if tokio::fs::metadata(&path).await.is_err() {
            return Err(ConfigError::NotFound);
        }
        let content = tokio::fs::read_to_string(&path).await?;
        let config = toml::from_str(&content)?;
        Ok(config)
    }

    pub async fn save_async(&self) -> Result<(), ConfigError> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        let content = toml::to_string_pretty(self)?;
        tokio::fs::write(&path, content).await?;
        Ok(())
    }

    /// Elimina el archivo de configuración del disco.
    ///
    /// El siguiente inicio detectará `ConfigError::NotFound`
    /// y lanzará el first_run wizard automáticamente.
    ///
    /// Si el archivo ya no existe devuelve `Ok(())` silenciosamente —
    /// el estado final deseado (no existe config) ya se cumple.
    pub async fn reset() -> Result<(), ConfigError> {
        let path = Self::config_path();
        match tokio::fs::remove_file(&path).await {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(ConfigError::Read(error)),
        }
    }

    pub fn has_subsystem_override(&self) -> bool {
        !self.subsystem.override_subsystem.is_empty()
    }

    pub fn to_executor_config(&self) -> crate::shell::executor::ExecutionConfig {
        let mut cfg = crate::shell::executor::ExecutionConfig::minimal();
        if self.execution.capture_output {
            cfg = cfg.with_capture_output();
        }
        if self.execution.capture_duration {
            cfg = cfg.with_capture_duration();
        }
        if self.execution.capture_command_trace {
            cfg = cfg.with_capture_command_trace();
        }
        if self.execution.timeout_secs > 0 {
            cfg = cfg.with_timeout(self.execution.timeout_secs);
        }
        cfg
    }
}
