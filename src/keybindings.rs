use std::{collections::HashMap, env, fs, path::PathBuf};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde::Deserialize;

const DEFAULT_CONFIG_TOML: &str = include_str!("../config/default-config.toml");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Exit,
    Evaluate,
    Backspace,
    DeleteWordBackward,
    CursorLeft,
    CursorRight,
    HistoryPrev,
    HistoryNext,
    ClearInput,
}

impl Action {
    fn from_id(action_id: &str) -> Option<Self> {
        match action_id {
            "app.exit" => Some(Self::Exit),
            "app.evaluate" => Some(Self::Evaluate),
            "app.backspace" => Some(Self::Backspace),
            "app.delete_word_backward" => Some(Self::DeleteWordBackward),
            "app.cursor_left" => Some(Self::CursorLeft),
            "app.cursor_right" => Some(Self::CursorRight),
            "history.prev" => Some(Self::HistoryPrev),
            "history.next" => Some(Self::HistoryNext),
            "app.clear_input" => Some(Self::ClearInput),
            _ => None,
        }
    }
}

#[derive(Debug, Deserialize)]
struct ConfigFile {
    keybindings: Option<HashMap<String, String>>,
}

#[derive(Debug, Default)]
pub struct KeyBindings {
    actions_by_key: HashMap<String, Action>,
}

impl KeyBindings {
    pub fn load() -> Self {
        let mut actions_by_key = HashMap::new();
        let default_config = parse_config(DEFAULT_CONFIG_TOML, "embedded default config");
        apply_bindings(
            &mut actions_by_key,
            default_config.keybindings,
            "embedded default",
        );

        if let Some(path) = user_config_path() {
            if path.exists() {
                match fs::read_to_string(&path) {
                    Ok(content) => {
                        let source = path.display().to_string();
                        let user_config = parse_config(&content, &source);
                        apply_bindings(&mut actions_by_key, user_config.keybindings, &source);
                    }
                    Err(error) => {
                        eprintln!("warning: failed to read {}: {error}", path.display());
                    }
                }
            }
        }

        Self { actions_by_key }
    }

    pub fn action_for_event(&self, key_event: KeyEvent) -> Option<Action> {
        key_event_to_id(key_event).and_then(|id| self.actions_by_key.get(&id).copied())
    }
}

fn parse_config(content: &str, source: &str) -> ConfigFile {
    match toml::from_str::<ConfigFile>(content) {
        Ok(config) => config,
        Err(error) => {
            eprintln!("warning: failed to parse {source}: {error}");
            ConfigFile { keybindings: None }
        }
    }
}

fn apply_bindings(
    actions_by_key: &mut HashMap<String, Action>,
    bindings: Option<HashMap<String, String>>,
    source: &str,
) {
    let Some(bindings) = bindings else {
        return;
    };

    for (key_spec, action_id) in bindings {
        let Some(key_id) = key_spec_to_id(&key_spec) else {
            eprintln!("warning: invalid keybinding key '{key_spec}' in {source}");
            continue;
        };

        if action_id == "none" {
            actions_by_key.remove(&key_id);
            continue;
        }

        let Some(action) = Action::from_id(&action_id) else {
            eprintln!("warning: unknown action '{action_id}' for key '{key_spec}' in {source}");
            continue;
        };

        actions_by_key.insert(key_id, action);
    }
}

fn key_event_to_id(key_event: KeyEvent) -> Option<String> {
    let base = match key_event.code {
        KeyCode::Backspace => "backspace".to_string(),
        KeyCode::Enter => "enter".to_string(),
        KeyCode::Esc => "esc".to_string(),
        KeyCode::Up => "up".to_string(),
        KeyCode::Down => "down".to_string(),
        KeyCode::Left => "left".to_string(),
        KeyCode::Right => "right".to_string(),
        KeyCode::PageUp => "pageup".to_string(),
        KeyCode::PageDown => "pagedown".to_string(),
        KeyCode::Delete => "delete".to_string(),
        KeyCode::Home => "home".to_string(),
        KeyCode::End => "end".to_string(),
        KeyCode::Tab => "tab".to_string(),
        KeyCode::BackTab => "backtab".to_string(),
        KeyCode::Char(character) => character.to_ascii_lowercase().to_string(),
        KeyCode::F(number) => format!("f{number}"),
        _ => return None,
    };

    Some(compose_key_id(base, key_event.modifiers))
}

fn key_spec_to_id(spec: &str) -> Option<String> {
    let mut modifiers = KeyModifiers::empty();
    let mut parts = spec
        .split('+')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(|part| part.to_ascii_lowercase())
        .collect::<Vec<_>>();

    if parts.is_empty() {
        return None;
    }

    let base = parts.pop()?;
    for modifier in parts {
        match modifier.as_str() {
            "ctrl" | "control" => modifiers.insert(KeyModifiers::CONTROL),
            "alt" => modifiers.insert(KeyModifiers::ALT),
            "shift" => modifiers.insert(KeyModifiers::SHIFT),
            _ => return None,
        }
    }

    let base = normalize_base_key(&base)?;
    Some(compose_key_id(base, modifiers))
}

fn normalize_base_key(base: &str) -> Option<String> {
    match base {
        "esc" | "escape" => Some("esc".to_string()),
        "enter" => Some("enter".to_string()),
        "backspace" => Some("backspace".to_string()),
        "up" => Some("up".to_string()),
        "down" => Some("down".to_string()),
        "left" => Some("left".to_string()),
        "right" => Some("right".to_string()),
        "pageup" | "page-up" => Some("pageup".to_string()),
        "pagedown" | "page-down" => Some("pagedown".to_string()),
        "home" => Some("home".to_string()),
        "end" => Some("end".to_string()),
        "delete" | "del" => Some("delete".to_string()),
        "tab" => Some("tab".to_string()),
        "backtab" => Some("backtab".to_string()),
        _ if base.starts_with('f') && base[1..].parse::<u8>().is_ok() => Some(base.to_string()),
        _ if base.chars().count() == 1 => Some(base.to_string()),
        _ => None,
    }
}

fn compose_key_id(base: String, modifiers: KeyModifiers) -> String {
    let mut prefixes = Vec::new();
    if modifiers.contains(KeyModifiers::CONTROL) {
        prefixes.push("ctrl");
    }
    if modifiers.contains(KeyModifiers::ALT) {
        prefixes.push("alt");
    }
    if modifiers.contains(KeyModifiers::SHIFT) {
        prefixes.push("shift");
    }

    if prefixes.is_empty() {
        return base;
    }

    format!("{}+{base}", prefixes.join("+"))
}

fn user_config_path() -> Option<PathBuf> {
    if let Some(config_home) = env::var_os("XDG_CONFIG_HOME") {
        return Some(PathBuf::from(config_home).join("lifocalc/config.toml"));
    }

    let home = env::var_os("HOME")?;
    Some(PathBuf::from(home).join(".config/lifocalc/config.toml"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_key_spec_for_ctrl_char() {
        assert_eq!(key_spec_to_id("ctrl+l"), Some("ctrl+l".to_string()));
    }

    #[test]
    fn parses_key_spec_for_named_keys() {
        assert_eq!(key_spec_to_id("PageUp"), Some("pageup".to_string()));
        assert_eq!(key_spec_to_id("Esc"), Some("esc".to_string()));
        assert_eq!(
            key_spec_to_id("ctrl+backspace"),
            Some("ctrl+backspace".to_string())
        );
    }

    #[test]
    fn parses_event_to_key_id() {
        let event = KeyEvent::new(KeyCode::Char('C'), KeyModifiers::CONTROL);
        assert_eq!(key_event_to_id(event), Some("ctrl+c".to_string()));
    }

    #[test]
    fn none_unbinds_default_key() {
        let mut actions = HashMap::new();
        apply_bindings(
            &mut actions,
            Some(HashMap::from([(
                String::from("up"),
                String::from("history.prev"),
            )])),
            "test",
        );
        apply_bindings(
            &mut actions,
            Some(HashMap::from([(String::from("up"), String::from("none"))])),
            "test",
        );

        assert!(!actions.contains_key("up"));
    }
}
