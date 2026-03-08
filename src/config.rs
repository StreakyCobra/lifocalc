use std::{collections::HashMap, env, fs, path::PathBuf};

use serde::Deserialize;

pub(crate) const DEFAULT_CONFIG_TOML: &str = include_str!("../config/default-config.toml");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ApproximationHintConfig {
    pub stack: bool,
    pub input: bool,
}

impl Default for ApproximationHintConfig {
    fn default() -> Self {
        Self {
            stack: true,
            input: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DisplayConfig {
    pub approximation_hint: ApproximationHintConfig,
}

impl DisplayConfig {
    pub fn load() -> Self {
        let mut display = Self::default();

        for (config, _) in load_config_layers() {
            apply_display_config(&mut display, config.display);
        }

        display
    }
}

#[derive(Debug, Deserialize, Default)]
pub(crate) struct ConfigFile {
    pub(crate) keybindings: Option<HashMap<String, String>>,
    pub(crate) display: Option<RawDisplayConfig>,
}

#[derive(Debug, Deserialize, Default)]
pub(crate) struct RawDisplayConfig {
    approximation_hint: Option<RawApproximationHintConfig>,
}

#[derive(Debug, Deserialize, Default)]
pub(crate) struct RawApproximationHintConfig {
    stack: Option<bool>,
    input: Option<bool>,
}

pub(crate) fn load_config_layers() -> Vec<(ConfigFile, String)> {
    let mut layers = vec![(
        parse_config(DEFAULT_CONFIG_TOML, "embedded default config"),
        "embedded default".to_string(),
    )];

    if let Some(path) = user_config_path() {
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(content) => {
                    let source = path.display().to_string();
                    layers.push((parse_config(&content, &source), source));
                }
                Err(error) => {
                    eprintln!("warning: failed to read {}: {error}", path.display());
                }
            }
        }
    }

    layers
}

fn parse_config(content: &str, source: &str) -> ConfigFile {
    match toml::from_str::<ConfigFile>(content) {
        Ok(config) => config,
        Err(error) => {
            eprintln!("warning: failed to parse {source}: {error}");
            ConfigFile::default()
        }
    }
}

fn apply_display_config(display: &mut DisplayConfig, raw: Option<RawDisplayConfig>) {
    let Some(raw) = raw else {
        return;
    };

    if let Some(approximation_hint) = raw.approximation_hint {
        if let Some(stack) = approximation_hint.stack {
            display.approximation_hint.stack = stack;
        }
        if let Some(input) = approximation_hint.input {
            display.approximation_hint.input = input;
        }
    }
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
    fn display_config_defaults_to_enabled_hints() {
        assert_eq!(
            DisplayConfig::default(),
            DisplayConfig {
                approximation_hint: ApproximationHintConfig {
                    stack: true,
                    input: true,
                },
            }
        );
    }

    #[test]
    fn display_config_applies_partial_override() {
        let mut display = DisplayConfig::default();
        apply_display_config(
            &mut display,
            Some(RawDisplayConfig {
                approximation_hint: Some(RawApproximationHintConfig {
                    stack: Some(false),
                    input: None,
                }),
            }),
        );

        assert_eq!(
            display,
            DisplayConfig {
                approximation_hint: ApproximationHintConfig {
                    stack: false,
                    input: true,
                },
            }
        );
    }
}
