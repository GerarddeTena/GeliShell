use crate::shell::config::ShellConfig;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GHistoryError {
    #[error("failed to read g history: {0}")]
    Read(#[from] std::io::Error),

    #[error("failed to parse g history: {0}")]
    Parse(#[from] toml::de::Error),

    #[error("failed to serialize g history: {0}")]
    Serialize(#[from] toml::ser::Error),
}

// ══════════════════════════════════════════════════════════════
// GEntry — una entrada del historial
// ══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GEntry {
    pub path: String,
    pub visits: u32,
    pub last_visit: u64, // Unix timestamp en segundos
}

impl GEntry {
    pub fn new(path: String) -> Self {
        Self {
            path,
            visits: 1,
            last_visit: super::frequency::now_secs(),
        }
    }

    /// Registra una nueva visita
    pub fn record_visit(&mut self) {
        self.visits += 1;
        self.last_visit = super::frequency::now_secs();
    }

    /// Score de frecency de esta entrada
    pub fn score(&self, case_bonus: f64) -> f64 {
        super::frequency::frecency_score(self.visits, self.last_visit, case_bonus)
    }
}

// ══════════════════════════════════════════════════════════════
// GHistory — colección persistida
// ══════════════════════════════════════════════════════════════

#[derive(Debug, Serialize, Deserialize, Default)]
struct GHistoryFile {
    #[serde(default)]
    entries: Vec<GEntry>,
}

pub struct GHistory {
    entries: Vec<GEntry>,
    path: PathBuf,
}

impl GHistory {
    /// Ruta del archivo de historial
    pub fn history_path() -> PathBuf {
        ShellConfig::config_path()
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."))
            .join("g_history.toml")
    }

    /// Carga desde disco — crea vacío si no existe
    pub fn load() -> Self {
        let path = Self::history_path();
        let entries = if path.exists() {
            std::fs::read_to_string(&path)
                .ok()
                .and_then(|s| toml::from_str::<GHistoryFile>(&s).ok())
                .map(|f| f.entries)
                .unwrap_or_default()
        } else {
            Vec::new()
        };

        Self { entries, path }
    }

    /// Persiste en disco
    pub fn save(&self) -> Result<(), GHistoryError> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let file = GHistoryFile {
            entries: self.entries.clone(),
        };
        let content = toml::to_string_pretty(&file)?;
        std::fs::write(&self.path, content)?;
        Ok(())
    }

    /// Registra una visita al directorio actual
    pub fn record_visit(&mut self, path: &str) {
        if let Some(entry) = self.entries.iter_mut().find(|e| e.path == path) {
            entry.record_visit();
        } else {
            self.entries.push(GEntry::new(path.to_owned()));
        }
        // Persiste en background — ignora errores silenciosamente
        self.save().ok();
    }

    /// Busca el mejor candidato para un patrón
    pub fn best_match(&self, pattern: &str) -> Option<&GEntry> {
        self.entries
            .iter()
            .filter_map(|entry| {
                super::matcher::match_pattern(&entry.path, pattern).map(|m| (entry, m.case_bonus))
            })
            .max_by(|(a, bonus_a), (b, bonus_b)| {
                let score_a = a.score(*bonus_a);
                let score_b = b.score(*bonus_b);
                score_a
                    .partial_cmp(&score_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(entry, _)| entry)
    }

    /// Top N entradas por score — para display
    pub fn top(&self, n: usize) -> Vec<(&GEntry, f64)> {
        let mut scored: Vec<(&GEntry, f64)> =
            self.entries.iter().map(|e| (e, e.score(0.0))).collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        scored.into_iter().take(n).collect()
    }

    /// Rutas recomendadas para autocompletado de `g`
    pub fn completion_candidates(&self, limit: usize) -> Vec<String> {
        self.top(limit)
            .into_iter()
            .map(|(entry, _)| entry.path.clone())
            .collect()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.save().ok();
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_history(entries: Vec<(&str, u32, u64)>) -> GHistory {
        let entries = entries
            .into_iter()
            .map(|(path, visits, last_visit)| GEntry {
                path: path.to_owned(),
                visits,
                last_visit,
            })
            .collect();

        GHistory {
            entries,
            path: PathBuf::from("/tmp/test_g_history.toml"),
        }
    }

    #[test]
    fn best_match_returns_highest_frecency() {
        let now = super::super::frequency::now_secs();
        let history = make_history(vec![
            ("/home/gerard/projects/geliShell", 50, now),
            ("/home/gerard/projects/other", 10, now),
        ]);

        let result = history.best_match("geli");
        assert!(result.is_some());
        assert!(result.unwrap().path.contains("geliShell"));
    }

    #[test]
    fn exact_case_wins_over_higher_frequency() {
        let now = super::super::frequency::now_secs();
        let history = make_history(vec![
            // Más frecuencia pero case insensitive match
            // (no suficiente para compensar el bonus de case exacto)
            ("/home/gerard/projects/gelishell", 8, now),
            // Baja frecuencia pero exact case match
            ("/home/gerard/projects/geliShell", 5, now),
        ]);

        let result = history.best_match("geliShell");
        assert!(result.is_some());
        // El exact case match debe ganar gracias al bonus de +50
        assert!(result.unwrap().path.contains("geliShell"));
    }

    #[test]
    fn top_returns_sorted_by_score() {
        let now = super::super::frequency::now_secs();
        let history = make_history(vec![("/a", 1, now), ("/b", 10, now), ("/c", 5, now)]);

        let top = history.top(3);
        assert_eq!(top[0].0.path, "/b");
        assert_eq!(top[1].0.path, "/c");
        assert_eq!(top[2].0.path, "/a");
    }

    #[test]
    fn record_visit_increments_existing() {
        let now = super::super::frequency::now_secs();
        let mut history = make_history(vec![("/home/gerard/projects/geliShell", 5, now)]);

        history.record_visit("/home/gerard/projects/geliShell");
        assert_eq!(history.entries[0].visits, 6);
    }

    #[test]
    fn record_visit_adds_new_entry() {
        let now = super::super::frequency::now_secs();
        let mut history = make_history(vec![("/existing", 5, now)]);

        history.record_visit("/new/path");
        assert_eq!(history.entries.len(), 2);
        assert_eq!(history.entries[1].path, "/new/path");
    }

    #[test]
    fn best_match_returns_none_for_no_match() {
        let now = super::super::frequency::now_secs();
        let history = make_history(vec![("/home/gerard/projects/geliShell", 5, now)]);

        assert!(history.best_match("xxxxxxxx").is_none());
    }
}
