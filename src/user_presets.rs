use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreset {
    pub emoji: String,
    pub label: String,
    pub prompt: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct UserPresets {
    #[serde(flatten)]
    pub presets: BTreeMap<String, UserPreset>,
}

fn presets_path() -> Result<PathBuf> {
    let home = dirs::home_dir().context("could not determine home directory")?;
    let dir = home.join(".vibeband");
    std::fs::create_dir_all(&dir)?;
    Ok(dir.join("presets.json"))
}

pub fn load() -> Result<UserPresets> {
    let path = presets_path()?;
    if !path.exists() {
        return Ok(UserPresets::default());
    }
    let data = std::fs::read_to_string(&path)?;
    let presets: UserPresets = serde_json::from_str(&data)?;
    Ok(presets)
}

pub fn save_all(presets: &UserPresets) -> Result<()> {
    let path = presets_path()?;
    let data = serde_json::to_string_pretty(presets)?;
    std::fs::write(&path, data)?;
    Ok(())
}

pub fn save(name: &str, emoji: &str, label: &str, prompt: &str) -> Result<()> {
    let mut presets = load()?;
    presets.presets.insert(
        name.to_string(),
        UserPreset {
            emoji: emoji.to_string(),
            label: label.to_string(),
            prompt: prompt.to_string(),
        },
    );
    save_all(&presets)
}

pub fn remove(name: &str) -> Result<bool> {
    let mut presets = load()?;
    let existed = presets.presets.remove(name).is_some();
    if existed {
        save_all(&presets)?;
    }
    Ok(existed)
}

pub fn find(name: &str) -> Result<Option<UserPreset>> {
    let presets = load()?;
    Ok(presets.presets.get(name).cloned())
}
