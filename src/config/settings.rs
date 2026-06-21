use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub const MIN_REFRESH_MS: u64 = 16;
pub const MAX_REFRESH_MS: u64 = 1000;
pub const DEFAULT_REFRESH_MS: u64 = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeName {
    Mizu,
    Abyss,
    Coral,
}

impl Default for ThemeName {
    fn default() -> Self {
        ThemeName::Mizu
    }
}

impl ThemeName {
    pub fn next(self) -> Self {
        match self {
            ThemeName::Mizu => ThemeName::Abyss,
            ThemeName::Abyss => ThemeName::Coral,
            ThemeName::Coral => ThemeName::Mizu,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default = "default_refresh_ms")]
    pub refresh_rate_ms: u64,
    #[serde(default)]
    pub theme: ThemeName,
    #[serde(default = "default_true")]
    pub flow_enabled: bool,
}

fn default_refresh_ms() -> u64 {
    DEFAULT_REFRESH_MS
}

fn default_true() -> bool {
    true
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            refresh_rate_ms: DEFAULT_REFRESH_MS,
            theme: ThemeName::default(),
            flow_enabled: true,
        }
    }
}

impl Settings {
    pub fn sanitized(mut self) -> Self {
        if self.refresh_rate_ms < MIN_REFRESH_MS {
            self.refresh_rate_ms = MIN_REFRESH_MS;
        }
        if self.refresh_rate_ms > MAX_REFRESH_MS {
            self.refresh_rate_ms = MAX_REFRESH_MS;
        }
        self
    }

    pub fn load() -> Self {
        match Self::load_from_path(&config_path()) {
            Ok(s) => s.sanitized(),
            Err(_) => Self::default(),
        }
    }

    fn load_from_path(path: &Path) -> anyhow::Result<Self> {
        match std::fs::read_to_string(path) {
            Ok(content) => toml::from_str(&content)
                .map_err(anyhow::Error::from)
                .map_err(|e| e.context(format!("failed to parse {}", path.display()))),
            Err(e) if e.kind() == ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(anyhow::Error::from(e))
                .map_err(|e| e.context(format!("failed to read {}", path.display()))),
        }
    }

    pub fn save(&self) -> anyhow::Result<()> {
        self.save_to_path(&config_path())
    }

    fn save_to_path(&self, path: &Path) -> anyhow::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        let temp_path = path.with_extension("toml.tmp");
        std::fs::write(&temp_path, content)?;
        std::fs::rename(&temp_path, path)?;
        Ok(())
    }
}

fn config_path() -> PathBuf {
    if let Some(dir) = std::env::var_os("MIZU_CONFIG_DIR") {
        return PathBuf::from(dir).join("settings.toml");
    }
    dirs::config_dir()
        .or_else(|| dirs::home_dir().map(|h| h.join(".config")))
        .unwrap_or_else(|| PathBuf::from("."))
        .join("mizu")
        .join("settings.toml")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn defaults_are_sane() {
        let s = Settings::default();
        assert_eq!(s.refresh_rate_ms, DEFAULT_REFRESH_MS);
        assert_eq!(s.theme, ThemeName::Mizu);
        assert!(s.flow_enabled);
    }

    #[test]
    fn sanitizes_out_of_range_refresh() {
        let s = Settings {
            refresh_rate_ms: 1,
            ..Settings::default()
        };
        assert_eq!(s.sanitized().refresh_rate_ms, MIN_REFRESH_MS);

        let s = Settings {
            refresh_rate_ms: 99999,
            ..Settings::default()
        };
        assert_eq!(s.sanitized().refresh_rate_ms, MAX_REFRESH_MS);
    }

    #[test]
    fn theme_cycles() {
        assert_eq!(ThemeName::Mizu.next(), ThemeName::Abyss);
        assert_eq!(ThemeName::Abyss.next(), ThemeName::Coral);
        assert_eq!(ThemeName::Coral.next(), ThemeName::Mizu);
    }

    #[test]
    fn save_load_roundtrip() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("nested").join("settings.toml");

        let original = Settings {
            refresh_rate_ms: 33,
            theme: ThemeName::Coral,
            flow_enabled: false,
        };
        original.save_to_path(&path).unwrap();

        let loaded = Settings::load_from_path(&path).unwrap().sanitized();
        assert_eq!(loaded.refresh_rate_ms, 33);
        assert_eq!(loaded.theme, ThemeName::Coral);
        assert!(!loaded.flow_enabled);
    }

    #[test]
    fn missing_file_returns_default() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("nope.toml");
        let loaded = Settings::load_from_path(&path).unwrap();
        assert_eq!(loaded, Settings::default());
    }

    #[test]
    fn invalid_toml_errors() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("bad.toml");
        std::fs::write(&path, "not = [valid").unwrap();
        let err = Settings::load_from_path(&path).unwrap_err();
        assert!(err.to_string().contains("failed to parse"));
    }

    #[test]
    fn partial_toml_uses_defaults() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("partial.toml");
        std::fs::write(&path, "theme = \"coral\"\n").unwrap();
        let loaded = Settings::load_from_path(&path).unwrap().sanitized();
        assert_eq!(loaded.theme, ThemeName::Coral);
        assert_eq!(loaded.refresh_rate_ms, DEFAULT_REFRESH_MS);
        assert!(loaded.flow_enabled);
    }
}
