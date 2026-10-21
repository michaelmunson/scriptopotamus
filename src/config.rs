use crate::paths;
use anyhow::{bail, Context, Result};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub autoload: bool,
    pub autocomplete: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            autoload: false,
            autocomplete: false,
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let path = resolve_config_file()?;
        if !path.exists() {
            return Ok(Self::default());
        }
        let text = fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let mut cfg = Self::default();
        for (idx, raw) in text.lines().enumerate() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let (key, value) = parse_pair(line).with_context(|| {
                format!("{}:{}: expected key=value", path.display(), idx + 1)
            })?;
            cfg.set(&key, &value)?;
        }
        Ok(cfg)
    }

    pub fn save(&self) -> Result<PathBuf> {
        let path = config_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }
        let body = format!(
            "autoload={}\nautocomplete={}\n",
            bool_str(self.autoload),
            bool_str(self.autocomplete)
        );
        fs::write(&path, body).with_context(|| format!("failed to write {}", path.display()))?;
        Ok(path)
    }

    pub fn set(&mut self, key: &str, value: &str) -> Result<()> {
        let parsed = parse_bool(value)?;
        match key {
            "autoload" => self.autoload = parsed,
            "autocomplete" => self.autocomplete = parsed,
            _ => bail!("unknown config key '{key}' (expected autoload or autocomplete)"),
        }
        Ok(())
    }

    pub fn display(&self) -> String {
        format!(
            "autoload={}\nautocomplete={}\n",
            bool_str(self.autoload),
            bool_str(self.autocomplete)
        )
    }
}

pub fn config_path() -> Result<PathBuf> {
    if let Some(path) = std::env::var_os("SCRIPPO_CONFIG") {
        return Ok(PathBuf::from(path));
    }
    Ok(paths::scrippo_home()?.join("config"))
}

/// Prefer `~/.scrippo/config`; fall back to the old XDG path if present.
fn resolve_config_file() -> Result<PathBuf> {
    let path = config_path()?;
    if path.exists() {
        return Ok(path);
    }
    if std::env::var_os("SCRIPPO_CONFIG").is_none() {
        if let Some(home) = std::env::var_os("HOME") {
            let legacy = PathBuf::from(home)
                .join(".config")
                .join("scrippo")
                .join("config");
            if legacy.exists() {
                return Ok(legacy);
            }
        }
    }
    Ok(path)
}

fn parse_pair(line: &str) -> Result<(String, String)> {
    let (key, value) = line
        .split_once('=')
        .context("expected key=value")?;
    let key = key.trim();
    let value = value.trim();
    if key.is_empty() {
        bail!("empty key");
    }
    Ok((key.to_string(), value.to_string()))
}

fn parse_bool(value: &str) -> Result<bool> {
    match value.to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => Ok(true),
        "false" | "0" | "no" | "off" => Ok(false),
        _ => bail!("invalid boolean '{value}' (expected true or false)"),
    }
}

fn bool_str(v: bool) -> &'static str {
    if v { "true" } else { "false" }
}

pub fn apply_configure(pairs: &[String]) -> Result<()> {
    let mut cfg = Config::load()?;
    if pairs.is_empty() {
        print!("{}", cfg.display());
        return Ok(());
    }
    for pair in pairs {
        let (key, value) = parse_pair(pair)?;
        cfg.set(&key, &value)?;
    }
    let path = cfg.save()?;
    print!("{}", cfg.display());
    eprintln!("wrote {}", path.display());
    if cfg.autoload {
        eprintln!("add this to your shell rc to enable autoload:");
        eprintln!("  eval \"$(scrippo --init)\"");
    }
    Ok(())
}
