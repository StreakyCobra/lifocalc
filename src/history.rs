use std::{
    env, fs,
    io::Write,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub struct HistoryStore {
    path: PathBuf,
}

impl HistoryStore {
    pub fn for_user() -> Option<Self> {
        if let Some(state_home) = env::var_os("XDG_STATE_HOME") {
            let path = PathBuf::from(state_home).join("postarity/history");
            return Some(Self { path });
        }

        let home = env::var_os("HOME")?;
        let path = PathBuf::from(home).join(".local/state/postarity/history");
        Some(Self { path })
    }

    pub fn load_entries(&self) -> Result<Vec<String>, String> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }

        let raw = fs::read_to_string(&self.path)
            .map_err(|error| format!("failed to read {}: {error}", self.path.display()))?;

        Ok(deduplicate_entries(
            raw.lines()
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .map(str::to_string)
                .collect(),
        ))
    }

    pub fn append_entry(&self, entry: &str) -> Result<(), String> {
        let mut entries = self.load_entries()?;
        if entries.iter().any(|existing| existing == entry) {
            return Ok(());
        }

        entries.push(entry.to_string());
        self.write_entries(&entries)
    }

    pub fn write_entries(&self, entries: &[String]) -> Result<(), String> {
        let parent = self
            .path
            .parent()
            .ok_or_else(|| format!("invalid history path: {}", self.path.display()))?;
        ensure_directory(parent)?;

        let mut file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&self.path)
            .map_err(|error| format!("failed to open {}: {error}", self.path.display()))?;

        for entry in deduplicate_entries(entries.to_vec()) {
            writeln!(file, "{entry}")
                .map_err(|error| format!("failed to write {}: {error}", self.path.display()))?;
        }

        Ok(())
    }
}

fn deduplicate_entries(entries: Vec<String>) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut deduped = Vec::new();

    for entry in entries {
        if seen.insert(entry.clone()) {
            deduped.push(entry);
        }
    }

    deduped
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn append_entry_avoids_duplicates() {
        let store = test_store("append_entry_avoids_duplicates");
        store
            .append_entry("1 2 +")
            .expect("first append should succeed");
        store
            .append_entry("1 2 +")
            .expect("duplicate append should succeed without writing");

        let entries = store.load_entries().expect("load should succeed");
        assert_eq!(entries, vec!["1 2 +"]);

        let _ = fs::remove_file(&store.path);
    }

    #[test]
    fn write_entries_normalizes_existing_duplicates() {
        let store = test_store("write_entries_normalizes_existing_duplicates");
        store
            .write_entries(&[
                String::from("2"),
                String::from("2"),
                String::from("3"),
                String::from("2"),
            ])
            .expect("write should succeed");

        let entries = store.load_entries().expect("load should succeed");
        assert_eq!(entries, vec!["2", "3"]);

        let _ = fs::remove_file(&store.path);
    }

    fn test_store(name: &str) -> HistoryStore {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be valid")
            .as_nanos();
        let path = env::temp_dir().join(format!("postarity-{name}-{unique}.history"));
        HistoryStore { path }
    }
}

fn ensure_directory(path: &Path) -> Result<(), String> {
    fs::create_dir_all(path)
        .map_err(|error| format!("failed to create {}: {error}", path.display()))
}
