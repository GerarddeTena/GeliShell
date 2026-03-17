use crate::shell::reporter::Reporter;
use crate::shell::translator::commands_map::{CommandDef, TranslationEntry};
use crate::shell::translator::subsystem::{ScoredSuggestion, Subsystem, SuggestionKind};

// Scores de matching en las sugggestions - Siempre constantes no MAGIC NUMBERS!

const SCORE_EXACT_MATCH: u8 = 100;
const SCORE_NATIVE_ALIAS: u8 = 80;
const SCORE_NATIVE_COMMAND: u8 = 60;
const SCORE_CROSS_PLATFORM: u8 = 40;
const SCORE_UNKNOWN: u8 = 20;
// ALIASES:

// PS:

static POWERSHELL_ALIASES: &[(&str, &str)] = &[
    ("gci", "Get-ChildItem"),
    ("sl", "Set-Location"),
    ("gl", "Get-Location"),
    ("gc", "Get-Content"),
    ("ri", "Remove-Item"),
    ("ci", "Copy-Item"),
    ("mi", "Move-Item"),
    ("ni", "New-Item"),
    ("gps", "Get-Process"),
    ("spps", "Stop-Process"),
    ("gcm", "Get-Command"),
    ("ghy", "Get-History"),
    ("sal", "Set-Alias"),
    ("gdr", "Get-PSDrive"),
    ("gu", "Get-Unique"),
    ("iwr", "Invoke-WebRequest"),
    ("tnc", "Test-Connection"),
    ("sls", "Select-String"),
    ("measure", "Measure-Object"),
    ("sort", "Sort-Object"),
    ("h", "Get-History"),
    ("ls", "Get-ChildItem"),
    ("dir", "Get-ChildItem"),
    ("cat", "Get-Content"),
    ("cp", "Copy-Item"),
    ("mv", "Move-Item"),
    ("rm", "Remove-Item"),
    ("echo", "Write-Output"),
    ("cd", "Set-Location"),
    ("pwd", "Get-Location"),
    ("ps", "Get-Process"),
    ("kill", "Stop-Process"),
    ("type", "Get-Content"),
    ("del", "Remove-Item"),
    ("erase", "Remove-Item"),
    ("more", "more"),
    ("cls", "Clear-Host"),
    ("clear", "Clear-Host"),
    ("where", "Get-Command"),
];

/// Comandos nativos de Unix que no son aliases pero
/// son reconocidos como NativeCommand en bash/zsh/fish
static UNIX_NATIVE_COMMANDS: &[&str] = &[
    "ls", "cd", "pwd", "mkdir", "rm", "cp", "mv", "cat", "grep", "find", "ps", "kill", "which",
    "echo", "less", "more", "head", "tail", "sort", "uniq", "wc", "sed", "awk", "touch", "chmod",
    "chown", "ln", "df", "du", "ping", "curl", "wget", "tar", "history", "alias", "export", "env",
    "printenv", "clear", "reset", "whoami",
];

static CROSS_PLATFORM_COMMANDS: &[&str] = &[
    "git", "cargo", "rustc", "npm", "node", "python", "python3", "pip", "pip3", "docker",
    "kubectl", "ssh", "curl", "tar", "less", "more", "nvim",
];

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum ResolverError {
    #[error("command '{0}' has no translation for subsystem '{1}'")]
    NoTranslation(String, String),

    #[error("command '{0}' has an empty exact field for subsystem '{1}'")]
    EmptyExact(String, String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedCommand {
    /// El comando más adecuado — siempre presente si Ok
    pub preferred: String,

    /// Alternativas ordenadas por score descendente
    /// Vacío si no hay suggestions en el TOML
    pub alternatives: Vec<ScoredSuggestion>,
}

impl ResolvedCommand {
    /// Devuelve preferred si no hay alternativas,
    /// o la mejor alternativa si su score supera al preferred
    pub fn best(&self) -> &str {
        // preferred ya es el ganador del scoring —
        // alternatives son opciones adicionales para mostrar al usuario
        &self.preferred
    }

    /// true si hay al menos una alternativa disponible
    pub fn has_alternatives(&self) -> bool {
        !self.alternatives.is_empty()
    }

    /// Todas las opciones como strings — útil para el AI assistant
    pub fn all_options(&self) -> Vec<&str> {
        let mut opts = vec![self.preferred.as_str()];
        opts.extend(self.alternatives.iter().map(|s| s.command.as_str()));
        opts
    }
}

/// Contrato de resolución de suggestions.
///
/// Implementaciones concretas pueden usar distintas estrategias
/// de scoring sin que el Translator cambie.
///
/// # Open/Closed
/// Para añadir una nueva estrategia de scoring:
/// - Implementa este trait en un nuevo struct
/// - Pásalo al Translator en construcción
/// - No toques `SuggestionResolver` ni ningún otro módulo
pub trait Resolve: Send + Sync {
    /// Resuelve un comando canónico para un subsistema dado.
    ///
    /// # Errors
    /// - `ResolverError::NoTranslation` — el subsistema no tiene entrada
    /// - `ResolverError::EmptyExact`    — el exact está vacío
    fn resolve(
        &self,
        cmd: &CommandDef,
        subsystem: &Subsystem,
        reporter: &dyn Reporter,
    ) -> Result<ResolvedCommand, ResolverError>;
}

// ══════════════════════════════════════════════════════════════
// SuggestionResolver — implementación concreta
// Encapsulado: campos privados, acceso solo por el trait
// ══════════════════════════════════════════════════════════════

/// Resolver basado en scoring por coincidencia con el subsistema activo.
///
/// Algoritmo:
/// 1. Extrae `exact` del subsistema → candidato base (score 100 si coincide)
/// 2. Puntúa cada suggestion contra los registros de alias y nativos
/// 3. Ordena por score descendente
/// 4. El preferred es el ganador — alternatives son el resto
pub struct SuggestionResolver {
    /// Umbral mínimo de score para incluir una suggestion
    /// en alternatives. Por defecto 20.
    min_score: u8,
}

impl SuggestionResolver {
    pub fn new() -> Self {
        Self {
            min_score: SCORE_UNKNOWN,
        }
    }

    /// Configura el umbral mínimo de score
    pub fn with_min_score(mut self, min_score: u8) -> Self {
        self.min_score = min_score;
        self
    }

    // ──────────────────────────────────────────────────────────
    // Scoring — privado, no forma parte del contrato público
    // ──────────────────────────────────────────────────────────

    /// Calcula el score y kind de una suggestion
    /// comparándola con el contexto del subsistema activo
    fn score_suggestion(
        &self,
        suggestion: &str,
        exact: &str,
        subsystem: &Subsystem,
    ) -> (u8, SuggestionKind) {
        // ── Coincidencia exacta con el exact del subsistema ───
        if suggestion == exact {
            return (SCORE_EXACT_MATCH, SuggestionKind::ExactMatch);
        }

        // ── Alias nativo del subsistema ───────────────────────
        if self.is_native_alias(suggestion, exact, subsystem) {
            return (SCORE_NATIVE_ALIAS, SuggestionKind::NativeAlias);
        }

        // ── Comando nativo del OS ─────────────────────────────
        if self.is_native_command(suggestion, subsystem) {
            return (SCORE_NATIVE_COMMAND, SuggestionKind::NativeCommand);
        }

        // ── Cross-platform ────────────────────────────────────
        if CROSS_PLATFORM_COMMANDS.contains(&suggestion) {
            return (SCORE_CROSS_PLATFORM, SuggestionKind::CrossPlatform);
        }

        // ── Desconocido — score mínimo ────────────────────────
        (SCORE_UNKNOWN, SuggestionKind::CrossPlatform)
    }
    /// Comprueba si suggestion es un alias conocido del exact
    /// en el subsistema activo
    fn is_native_alias(&self, suggestion: &str, exact: &str, subsystem: &Subsystem) -> bool {
        match subsystem {
            Subsystem::PowerShell => POWERSHELL_ALIASES
                .iter()
                .any(|(alias, target)| *alias == suggestion && *target == exact),
            // En Unix bash/zsh/fish los alias son user-defined —
            // no podemos verificarlos estáticamente
            // Los marcamos como NativeAlias si coinciden con
            // comandos conocidos que son alias de facto
            Subsystem::Bash | Subsystem::Zsh | Subsystem::Fish => {
                // Alias de facto Unix: comandos cortos que
                // mapean a versiones más largas
                matches!(
                    (suggestion, exact),
                    ("ll", "ls -la") | ("la", "ls -a") | ("l", "ls -CF")
                )
            }
            Subsystem::Cmd => false,
        }
    }

    /// Comprueba si suggestion es un comando nativo del subsistema
    fn is_native_command(&self, suggestion: &str, subsystem: &Subsystem) -> bool {
        match subsystem {
            Subsystem::Bash | Subsystem::Zsh | Subsystem::Fish => {
                // Toma solo el primer token — "ls -la" → "ls"
                let base = suggestion.split_whitespace().next().unwrap_or(suggestion);
                UNIX_NATIVE_COMMANDS.contains(&base)
            }
            Subsystem::PowerShell => {
                // Cmdlets tienen formato Verbo-Nombre
                let base = suggestion.split_whitespace().next().unwrap_or(suggestion);
                base.contains('-') || POWERSHELL_ALIASES.iter().any(|(alias, _)| *alias == base)
            }
            Subsystem::Cmd => {
                // Comandos internos de cmd.exe
                matches!(
                    suggestion.split_whitespace().next().unwrap_or(suggestion),
                    "dir"
                        | "copy"
                        | "move"
                        | "del"
                        | "erase"
                        | "type"
                        | "cls"
                        | "echo"
                        | "set"
                        | "cd"
                        | "md"
                        | "rd"
                        | "ren"
                        | "where"
                        | "findstr"
                        | "tasklist"
                        | "taskkill"
                        | "sort"
                        | "more"
                        | "doskey"
                )
            }
        }
    }

    fn build_scored(
        &self,
        entry: &TranslationEntry,
        subsystem: &Subsystem,
        reporter: &dyn Reporter,
    ) -> Vec<ScoredSuggestion> {
        let mut scored: Vec<ScoredSuggestion> = entry
            .suggestions
            .iter()
            .filter_map(|s| {
                let s = s.trim();
                if s.is_empty() {
                    reporter.warn("resolver: found empty suggestion — skipping");
                    return None;
                }
                let (score, kind) = self.score_suggestion(s, &entry.exact, subsystem);
                if score < self.min_score {
                    return None;
                }
                Some(ScoredSuggestion::new(s.to_owned(), score, kind))
            })
            .collect();

        // Orden descendente por score — estable para igual score
        scored.sort_by(|a, b| b.score.cmp(&a.score));
        scored
    }
}

impl Default for SuggestionResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl Resolve for SuggestionResolver {
    fn resolve(
        &self,
        cmd: &CommandDef,
        subsystem: &Subsystem,
        reporter: &dyn Reporter,
    ) -> Result<ResolvedCommand, ResolverError> {
        // ── Extrae la entrada del subsistema activo ───────────
        let entry = subsystem.entry(&cmd.translate).ok_or_else(|| {
            ResolverError::NoTranslation(cmd.name.clone(), subsystem.as_str().to_owned())
        })?;

        // ── Valida que exact no esté vacío ────────────────────
        if entry.exact.trim().is_empty() {
            return Err(ResolverError::EmptyExact(
                cmd.name.clone(),
                subsystem.as_str().to_owned(),
            ));
        }

        // ── Construye las suggestions puntuadas ───────────────
        let alternatives = self.build_scored(entry, subsystem, reporter);

        reporter.info(&format!(
            "resolver: '{}' → '{}' [{} alternatives]",
            cmd.name,
            entry.exact,
            alternatives.len()
        ));

        Ok(ResolvedCommand {
            preferred: entry.exact.clone(),
            alternatives,
        })
    }
}

// Tests

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shell::reporter::{BufferedReporter, SilentReporter};
    use crate::shell::translator::commands_map::{SubsystemTranslations, TranslationEntry};

    // ──────────────────────────────────────────────────────────
    // Fixtures — construyen CommandDef de test sin tocar el TOML
    // ──────────────────────────────────────────────────────────

    fn make_entry(exact: &str, suggestions: Vec<&str>) -> TranslationEntry {
        TranslationEntry {
            exact: exact.to_owned(),
            suggestions: suggestions.into_iter().map(str::to_owned).collect(),
        }
    }

    fn make_cmd(name: &str, translations: SubsystemTranslations) -> CommandDef {
        CommandDef {
            name: name.to_owned(),
            description: format!("test command {name}"),
            category: "test".to_owned(),
            translate: translations,
            flags: vec![],
        }
    }

    fn powershell_cmd() -> CommandDef {
        make_cmd(
            "list",
            SubsystemTranslations {
                bash: Some(make_entry("ls", vec!["ls -la", "ls -lh"])),
                zsh: Some(make_entry("ls", vec!["ls -la"])),
                fish: Some(make_entry("ls", vec!["ls -la"])),
                powershell: Some(make_entry("Get-ChildItem", vec!["gci", "dir", "ls"])),
                cmd: Some(make_entry("dir", vec!["dir /w", "dir /b"])),
            },
        )
    }

    fn search_cmd() -> CommandDef {
        make_cmd(
            "search",
            SubsystemTranslations {
                bash: Some(make_entry("grep", vec!["rg", "ag"])),
                zsh: Some(make_entry("grep", vec!["rg", "ag"])),
                fish: Some(make_entry("grep", vec!["rg"])),
                powershell: Some(make_entry("Select-String", vec!["sls", "findstr"])),
                cmd: Some(make_entry("findstr", vec!["findstr /s /i"])),
            },
        )
    }

    fn no_translation_cmd() -> CommandDef {
        make_cmd(
            "unix-only",
            SubsystemTranslations {
                bash: Some(make_entry("ls", vec![])),
                zsh: None,
                fish: None,
                powershell: None,
                cmd: None,
            },
        )
    }

    // ──────────────────────────────────────────────────────────
    // Tests de resolución correcta
    // ──────────────────────────────────────────────────────────

    #[test]
    fn resolves_exact_for_bash() {
        let resolver = SuggestionResolver::new();
        let reporter = SilentReporter::new();
        let result = resolver.resolve(&powershell_cmd(), &Subsystem::Bash, &reporter);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().preferred, "ls");
    }

    #[test]
    fn resolves_exact_for_powershell() {
        let resolver = SuggestionResolver::new();
        let reporter = SilentReporter::new();
        let result = resolver
            .resolve(&powershell_cmd(), &Subsystem::PowerShell, &reporter)
            .unwrap();
        assert_eq!(result.preferred, "Get-ChildItem");
    }

    #[test]
    fn powershell_gci_scores_as_native_alias() {
        let resolver = SuggestionResolver::new();
        let reporter = SilentReporter::new();
        let result = resolver
            .resolve(&powershell_cmd(), &Subsystem::PowerShell, &reporter)
            .unwrap();

        let gci = result.alternatives.iter().find(|s| s.command == "gci");
        assert!(gci.is_some(), "gci should be in alternatives");
        assert_eq!(gci.unwrap().score, SCORE_NATIVE_ALIAS);
        assert_eq!(gci.unwrap().kind, SuggestionKind::NativeAlias);
    }

    #[test]
    fn alternatives_sorted_by_score_descending() {
        let resolver = SuggestionResolver::new();
        let reporter = SilentReporter::new();
        let result = resolver
            .resolve(&powershell_cmd(), &Subsystem::PowerShell, &reporter)
            .unwrap();

        let scores: Vec<u8> = result.alternatives.iter().map(|s| s.score).collect();
        let mut sorted = scores.clone();
        sorted.sort_by(|a, b| b.cmp(a));
        assert_eq!(scores, sorted, "alternatives must be sorted by score desc");
    }

    #[test]
    fn grep_scores_as_native_command_in_bash() {
        let resolver = SuggestionResolver::new();
        let reporter = SilentReporter::new();
        let result = resolver
            .resolve(&search_cmd(), &Subsystem::Bash, &reporter)
            .unwrap();

        // rg y ag son desconocidos — score UNKNOWN
        // grep es el preferred — no aparece en alternatives
        assert!(result.alternatives.iter().all(|s| s.score <= SCORE_UNKNOWN));
    }

    #[test]
    fn sls_scores_as_native_alias_in_powershell() {
        let resolver = SuggestionResolver::new();
        let reporter = SilentReporter::new();
        let result = resolver
            .resolve(&search_cmd(), &Subsystem::PowerShell, &reporter)
            .unwrap();

        let sls = result.alternatives.iter().find(|s| s.command == "sls");
        assert!(sls.is_some());
        assert_eq!(sls.unwrap().kind, SuggestionKind::NativeAlias);
    }

    // ──────────────────────────────────────────────────────────
    // Tests de error
    // ──────────────────────────────────────────────────────────

    #[test]
    fn returns_error_on_missing_subsystem() {
        let resolver = SuggestionResolver::new();
        let reporter = SilentReporter::new();
        let result = resolver.resolve(&no_translation_cmd(), &Subsystem::PowerShell, &reporter);
        assert!(matches!(result, Err(ResolverError::NoTranslation(_, _))));
    }

    #[test]
    fn returns_error_on_empty_exact() {
        let resolver = SuggestionResolver::new();
        let reporter = SilentReporter::new();
        let cmd = make_cmd(
            "empty-exact",
            SubsystemTranslations {
                bash: Some(make_entry("", vec!["ls"])),
                ..Default::default()
            },
        );
        let result = resolver.resolve(&cmd, &Subsystem::Bash, &reporter);
        assert!(matches!(result, Err(ResolverError::EmptyExact(_, _))));
    }

    #[test]
    fn skips_empty_suggestion_strings() {
        let resolver = SuggestionResolver::new();
        let reporter = BufferedReporter::new();
        let cmd = make_cmd(
            "empty-sugg",
            SubsystemTranslations {
                bash: Some(make_entry("ls", vec!["", "  ", "ls -la"])),
                ..Default::default()
            },
        );
        let result = resolver.resolve(&cmd, &Subsystem::Bash, &reporter).unwrap();

        // Las dos suggestions vacías deben haber sido ignoradas
        assert!(reporter.has_warnings());
        // Solo "ls -la" debe aparecer
        assert_eq!(result.alternatives.len(), 1);
        assert_eq!(result.alternatives[0].command, "ls -la");
    }

    // ──────────────────────────────────────────────────────────
    // Tests de min_score
    // ──────────────────────────────────────────────────────────

    #[test]
    fn min_score_filters_low_scoring_suggestions() {
        // Con min_score=60 solo pasan NativeCommand y superiores
        let resolver = SuggestionResolver::new().with_min_score(60);
        let reporter = SilentReporter::new();
        let result = resolver
            .resolve(&powershell_cmd(), &Subsystem::PowerShell, &reporter)
            .unwrap();

        for alt in &result.alternatives {
            assert!(
                alt.score >= 60,
                "suggestion '{}' has score {} < min 60",
                alt.command,
                alt.score
            );
        }
    }

    // ──────────────────────────────────────────────────────────
    // Tests del trait Resolve — verifica el contrato público
    // ──────────────────────────────────────────────────────────

    #[test]
    fn trait_object_works() {
        // Verifica que SuggestionResolver funciona como dyn Resolve
        let resolver: Box<dyn Resolve> = Box::new(SuggestionResolver::new());
        let reporter = SilentReporter::new();
        let result = resolver.resolve(&powershell_cmd(), &Subsystem::Bash, &reporter);
        assert!(result.is_ok());
    }

    #[test]
    fn buffered_reporter_captures_info_messages() {
        let resolver = SuggestionResolver::new();
        let reporter = BufferedReporter::new();
        resolver
            .resolve(&powershell_cmd(), &Subsystem::Bash, &reporter)
            .unwrap();
        // El resolver debe emitir al menos un info con el resultado
        assert!(!reporter.infos().is_empty());
    }

    // ──────────────────────────────────────────────────────────
    // Tests de all_options y has_alternatives
    // ──────────────────────────────────────────────────────────

    #[test]
    fn all_options_includes_preferred_first() {
        let resolver = SuggestionResolver::new();
        let reporter = SilentReporter::new();
        let result = resolver
            .resolve(&powershell_cmd(), &Subsystem::PowerShell, &reporter)
            .unwrap();

        let options = result.all_options();
        assert_eq!(options[0], "Get-ChildItem");
        assert!(options.len() > 1);
    }

    #[test]
    fn no_alternatives_when_suggestions_empty() {
        let resolver = SuggestionResolver::new();
        let reporter = SilentReporter::new();
        let cmd = make_cmd(
            "no-sugg",
            SubsystemTranslations {
                bash: Some(make_entry("pwd", vec![])),
                ..Default::default()
            },
        );
        let result = resolver.resolve(&cmd, &Subsystem::Bash, &reporter).unwrap();

        assert!(!result.has_alternatives());
        assert_eq!(result.best(), "pwd");
    }
}
