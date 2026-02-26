use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::path::PathBuf;

const MODEL_ID: &str = "eleven_text_to_sound_v2";
const DURATION: u32 = 30;

pub fn cache_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().context("could not determine home directory")?;
    let dir = home.join(".vibeband").join("cache");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn cache_key(prompt: &str) -> String {
    let input = format!("{prompt}|{DURATION}|loop=true|{MODEL_ID}");
    let hash = Sha256::digest(input.as_bytes());
    hex::encode(&hash[..8]) // first 16 hex chars
}

pub fn cache_path(prompt: &str) -> Result<PathBuf> {
    let dir = cache_dir()?;
    let key = cache_key(prompt);
    Ok(dir.join(format!("{key}.mp3")))
}

pub fn read_cache(prompt: &str) -> Result<Option<Vec<u8>>> {
    let path = cache_path(prompt)?;
    if path.exists() {
        let data = std::fs::read(&path)?;
        Ok(Some(data))
    } else {
        Ok(None)
    }
}

pub fn write_cache(prompt: &str, data: &[u8]) -> Result<()> {
    let path = cache_path(prompt)?;
    std::fs::write(&path, data)?;
    Ok(())
}
