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
            let path = PathBuf::from(state_home).join("lifocalc/history");
            return Some(Self { path });
        }

        let home = env::var_os("HOME")?;
        let path = PathBuf::from(home).join(".local/state/lifocalc/history");
        Some(Self { path })
    }

    pub fn load_entries(&self) -> Result<Vec<String>, String> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }

        let raw = fs::read_to_string(&self.path)
            .map_err(|error| format!("failed to read {}: {error}", self.path.display()))?;

        Ok(raw
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(str::to_string)
            .collect())
    }

    pub fn append_entry(&self, entry: &str) -> Result<(), String> {
        let parent = self
            .path
            .parent()
            .ok_or_else(|| format!("invalid history path: {}", self.path.display()))?;
        ensure_directory(parent)?;

        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|error| format!("failed to open {}: {error}", self.path.display()))?;

        writeln!(file, "{entry}")
            .map_err(|error| format!("failed to write {}: {error}", self.path.display()))
    }
}

fn ensure_directory(path: &Path) -> Result<(), String> {
    fs::create_dir_all(path)
        .map_err(|error| format!("failed to create {}: {error}", path.display()))
}
