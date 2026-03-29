use crate::shell::reporter::Reporter;
use crate::shell::translator::commands_map::{FlagDef, SubsystemTranslations, TranslationEntry};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Subsystem {
    Bash,
    Zsh,
    Fish,
    PowerShell,
    Cmd,
}

impl Subsystem {
    // Detecta el subsistema activo con prioridad decreciente:
    //
    // 1. `GELI_SUBSYSTEM` — override explícito del usuario
    // 2. `SHELL`          — variable heredada del OS
    // 3. `#[cfg]`         — default de compilación por plataforma

    pub fn detect(reporter: &dyn Reporter) -> Self {
        if let Ok(val) = std::env::var("GELI_SUBSYSTEM") {
            match Self::from_str(val.trim()) {
                Some(s) => {
                    reporter.info(&format!("subsystem: using GELI_SUBSYSTEM='{}'", val.trim()));
                    if !s.is_supported_on_platform() {
                        reporter.warn(&format!(
                            "subsystem '{}' has limited support on this platform — \
                             commands may not execute correctly",
                            s.as_str()
                        ));
                    }
                    return s;
                }
                None => {
                    reporter.warn(&format!(
                        "GELI_SUBSYSTEM='{}' is not a valid subsystem — \
                         falling back to auto-detect",
                        val.trim()
                    ));
                }
            }
        }

        #[cfg(not(target_os = "windows"))]
        if let Ok(shell) = std::env::var("SHELL") {
            if let Some(s) = Self::from_shell_path(&shell) {
                reporter.info(&format!("subsystem: detected from $SHELL='{shell}'"));
                return s;
            }
        }
        let default = Self::platform_default();
        reporter.info(&format!(
            "subsystem: using platform default '{}'",
            default.as_str()
        ));
        default
    }

    pub fn is_supported_on_platform(&self) -> bool {
        #[cfg(target_os = "windows")]
        if matches!(self, Self::Fish) {
            return false;
        }
        true
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "bash" => Some(Self::Bash),
            "zsh" => Some(Self::Zsh),
            "fish" => Some(Self::Fish),
            "powershell" | "pwsh" => Some(Self::PowerShell),
            "cmd" | "cmd.exe" => Some(Self::Cmd),
            _ => None,
        }
    }

    fn from_shell_path(path: &str) -> Option<Self> {
        let normalized = path.replace('\\', "/");
        let exe = normalized.rsplit('/').next().unwrap_or(path);
        let name = exe.strip_suffix(".exe").unwrap_or(exe).to_lowercase();

        match name.as_str() {
            "bash" | "git-bash" => Some(Self::Bash),
            "zsh" => Some(Self::Zsh),
            "fish" => Some(Self::Fish),
            "pwsh" | "powershell" => Some(Self::PowerShell),
            _ => None,
        }
    }

    #[cfg(target_os = "windows")]
    fn platform_default() -> Self {
        Self::PowerShell
    }

    #[cfg(not(target_os = "windows"))]
    fn platform_default() -> Self {
        Self::Bash
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Bash => "bash",
            Self::Zsh => "zsh",
            Self::Fish => "fish",
            Self::PowerShell => "powershell",
            Self::Cmd => "cmd",
        }
    }

    pub fn is_unix(&self) -> bool {
        matches!(self, Self::Bash | Self::Zsh | Self::Fish)
    }

    pub fn is_windows(&self) -> bool {
        matches!(self, Self::PowerShell | Self::Cmd)
    }

    pub fn entry<'a>(
        &self,
        translations: &'a SubsystemTranslations,
    ) -> Option<&'a TranslationEntry> {
        match self {
            Self::Bash => translations.bash.as_ref(),
            Self::Zsh => translations.zsh.as_ref(),
            Self::Fish => translations.fish.as_ref(),
            Self::PowerShell => translations.powershell.as_ref(),
            Self::Cmd => translations.cmd.as_ref(),
        }
    }

    pub fn flag<'a>(&self, flag: &'a FlagDef) -> Option<&'a str> {
        match self {
            Self::Bash => flag.bash.as_deref(),
            Self::Zsh => flag.zsh.as_deref(),
            Self::Fish => flag.fish.as_deref(),
            Self::PowerShell => flag.powershell.as_deref(),
            Self::Cmd => flag.cmd.as_deref(),
        }
    }

    pub fn and_operator(&self) -> &'static str {
        match self {
            Self::Cmd => " & ", // cmd no tiene && real
            _ => " && ",
        }
    }

    pub fn or_operator(&self) -> &'static str {
        match self {
            Self::Cmd => " & ", // cmd no tiene || real
            _ => " || ",
        }
    }

    pub fn sequence_operator(&self) -> &'static str {
        match self {
            Self::Cmd => " & ",
            Self::PowerShell => " ; ",
            _ => " ; ",
        }
    }

    pub fn background_wrap(&self, cmd: &str) -> String {
        match self {
            Self::PowerShell => format!("Start-Process {cmd}"),
            Self::Cmd => format!("start {cmd}"),
            _ => format!("{cmd} &"),
        }
    }

    pub fn variable_syntax(&self, name: &str) -> String {
        match self {
            Self::PowerShell => format!("$env:{name}"),
            Self::Cmd => format!("%{name}%"),
            _ => format!("${name}"),
        }
    }
}

impl std::fmt::Display for Subsystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScoredSuggestion {
    pub command: String,
    pub score: u8,
    pub kind: SuggestionKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SuggestionKind {
    ExactMatch,
    NativeAlias,
    NativeCommand,
    CrossPlatform,
}

impl ScoredSuggestion {
    pub fn new(command: String, score: u8, kind: SuggestionKind) -> Self {
        Self {
            command,
            score,
            kind,
        }
    }
}

impl std::fmt::Display for ScoredSuggestion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} [score={}, kind={:?}]",
            self.command, self.score, self.kind
        )
    }
}

// ________TESTING_________
#[cfg(test)]
mod tests {
    use super::*;
    use crate::shell::reporter::SilentReporter;

    #[test]
    fn from_str_case_insensitive() {
        assert_eq!(Subsystem::from_str("BASH"), Some(Subsystem::Bash));
        assert_eq!(
            Subsystem::from_str("PowerShell"),
            Some(Subsystem::PowerShell)
        );
        assert_eq!(Subsystem::from_str("pwsh"), Some(Subsystem::PowerShell));
        assert_eq!(Subsystem::from_str("unknown"), None);
    }

    #[test]
    fn from_shell_path_parses_correctly() {
        assert_eq!(
            Subsystem::from_shell_path("/usr/bin/zsh"),
            Some(Subsystem::Zsh)
        );
        assert_eq!(
            Subsystem::from_shell_path("/bin/bash"),
            Some(Subsystem::Bash)
        );
        assert_eq!(
            Subsystem::from_shell_path("/usr/local/bin/fish"),
            Some(Subsystem::Fish)
        );
        assert_eq!(Subsystem::from_shell_path("/usr/bin/sh"), None);
    }

    #[test]
    fn operators_differ_by_subsystem() {
        assert_eq!(Subsystem::Bash.and_operator(), " && ");
        assert_eq!(Subsystem::Cmd.and_operator(), " & ");
        assert_eq!(Subsystem::PowerShell.or_operator(), " || ");
        assert_eq!(Subsystem::Cmd.or_operator(), " & ");
    }

    #[test]
    fn background_wrap_by_subsystem() {
        assert_eq!(
            Subsystem::Bash.background_wrap("cargo build"),
            "cargo build &"
        );
        assert_eq!(
            Subsystem::PowerShell.background_wrap("cargo build"),
            "Start-Process cargo build"
        );
        assert_eq!(
            Subsystem::Cmd.background_wrap("cargo build"),
            "start cargo build"
        );
    }

    #[test]
    fn variable_syntax_by_subsystem() {
        assert_eq!(Subsystem::Bash.variable_syntax("HOME"), "$HOME");
        assert_eq!(Subsystem::PowerShell.variable_syntax("HOME"), "$env:HOME");
        assert_eq!(Subsystem::Cmd.variable_syntax("HOME"), "%HOME%");
    }

    #[test]
    fn detect_respects_geli_subsystem_env() {
        unsafe {
            std::env::set_var("GELI_SUBSYSTEM", "fish");
        }
        let reporter = SilentReporter::new();
        let subsystem = Subsystem::detect(&reporter);
        unsafe {
            std::env::remove_var("GELI_SUBSYSTEM");
        }
        assert_eq!(subsystem, Subsystem::Fish);
    }

    #[test]
    fn detect_warns_on_invalid_geli_subsystem() {
        use crate::shell::reporter::BufferedReporter;
        unsafe {
            std::env::set_var("GELI_SUBSYSTEM", "invalid_shell");
        }
        let reporter = BufferedReporter::new();
        Subsystem::detect(&reporter);
        unsafe {
            std::env::remove_var("GELI_SUBSYSTEM");
        }
        assert!(reporter.has_warnings());
    }

    #[test]
    fn unix_windows_classification() {
        assert!(Subsystem::Bash.is_unix());
        assert!(Subsystem::Zsh.is_unix());
        assert!(Subsystem::Fish.is_unix());
        assert!(Subsystem::PowerShell.is_windows());
        assert!(Subsystem::Cmd.is_windows());
    }
}
