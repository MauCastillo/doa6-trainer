//! User preferences saved between runs (currently only the interface language).
//!
//! Stored as `key=value` lines in `%APPDATA%\doa6-trainer\settings.ini` on Windows
//! (also under Wine) or `$XDG_CONFIG_HOME/doa6-trainer/settings.ini` elsewhere.

use std::fs;
use std::path::PathBuf;

use super::i18n::Language;

const LANGUAGE_KEY: &str = "language";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Settings {
    pub language: Language,
}

impl Settings {
    /// Loads saved settings, falling back to the system language.
    pub fn load() -> Self {
        let saved = settings_path().and_then(|path| fs::read_to_string(path).ok()).and_then(|content| parse(&content));
        Self { language: saved.unwrap_or_else(Language::detect) }
    }

    /// Saves the settings. Failing to save is not fatal: the choice only lasts this session.
    pub fn save(&self) {
        let Some(path) = settings_path() else { return };
        if let Some(dir) = path.parent() {
            let _ = fs::create_dir_all(dir);
        }
        let _ = fs::write(path, format!("{LANGUAGE_KEY}={}\n", self.language.code()));
    }
}

fn parse(content: &str) -> Option<Language> {
    content
        .lines()
        .filter_map(|line| line.split_once('='))
        .find(|(key, _)| key.trim() == LANGUAGE_KEY)
        .and_then(|(_, value)| Language::from_code(value))
}

fn settings_path() -> Option<PathBuf> {
    let base = if cfg!(windows) {
        std::env::var_os("APPDATA").map(PathBuf::from)
    } else {
        std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
    };
    base.map(|dir| dir.join("doa6-trainer").join("settings.ini"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_saved_language() {
        assert_eq!(parse("language=es\n"), Some(Language::Spanish));
        assert_eq!(parse("other=1\r\nlanguage = en\r\n"), Some(Language::English));
        assert_eq!(parse(""), None);
    }
}
