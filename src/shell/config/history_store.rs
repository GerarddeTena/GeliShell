use super::ShellConfig;
use std::path::PathBuf;
use thiserror::Error;
use tokio::io::AsyncWriteExt;

#[derive(Debug, Error)]
pub enum HistoryStoreError {
    #[error("failed to read history: {0}")]
    Read(#[from] std::io::Error),
}

#[derive(Debug, Clone)]
pub struct PersistentCommandHistory {
    entries: Vec<String>,
    path: PathBuf,
}

impl Default for PersistentCommandHistory {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            path: ShellConfig::command_history_path(),
        }
    }
}

impl PersistentCommandHistory {
    pub async fn load_async() -> Result<Self, HistoryStoreError> {
        let path = ShellConfig::command_history_path();
        if tokio::fs::metadata(&path).await.is_err() {
            if let Some(parent) = path.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }
            tokio::fs::write(&path, "").await?;
            return Ok(Self {
                entries: Vec::new(),
                path,
            });
        }

        let content = tokio::fs::read_to_string(&path).await?;
        let entries = content
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(str::to_owned)
            .collect();
        Ok(Self { entries, path })
    }

    pub fn entries(&self) -> &[String] {
        &self.entries
    }

    pub async fn append_async(&mut self, raw_command: &str) -> Result<bool, HistoryStoreError> {
        let command = raw_command.trim();
        if command.is_empty() {
            return Ok(false);
        }

        if self.entries.last().is_some_and(|last| last == command) {
            return Ok(false);
        }

        if let Some(parent) = self.path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .await?;
        file.write_all(command.as_bytes()).await?;
        file.write_all(b"\n").await?;

        self.entries.push(command.to_owned());
        Ok(true)
    }

    #[cfg(test)]
    fn test_with_path(path: impl AsRef<std::path::Path>, entries: Vec<String>) -> Self {
        Self {
            entries,
            path: path.as_ref().to_path_buf(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn append_rejects_empty_command() {
        let base = std::env::temp_dir().join("geli_shell_history_test_empty.txt");
        let mut history = PersistentCommandHistory::test_with_path(base, Vec::new());
        let result = history.append_async("   ").await.unwrap();
        assert!(!result);
        assert!(history.entries().is_empty());
    }

    #[tokio::test]
    async fn append_rejects_consecutive_duplicates() {
        let base = std::env::temp_dir().join("geli_shell_history_test_dup.txt");
        let mut history = PersistentCommandHistory::test_with_path(base, vec!["ls".to_owned()]);
        let result = history.append_async("ls").await.unwrap();
        assert!(!result);
        assert_eq!(history.entries().len(), 1);
    }
}
