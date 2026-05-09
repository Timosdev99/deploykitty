use std::fs;
use std::path::PathBuf;

use color_eyre::eyre::{Result, eyre};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub key_path: String,
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            name: String::new(),
            host: String::new(),
            port: 22,
            username: String::from("root"),
            key_path: format!("{}/.ssh/id_rsa", std::env::var("HOME").unwrap_or_default()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Database {
    Postgres,
    Redis,
    MongoDB,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AiAgent {
    Hermes,
    OpenClaw,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReverseProxy {
    Caddy,
    Nginx,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeploymentTarget {
    Hardening,
    Database(Database),
    AiAgent(AiAgent),
    ReverseProxy(ReverseProxy),
    DockerCompose,
    Binary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preset {
    pub name: String,
    pub targets: Vec<DeploymentTarget>,
}

fn config_dir() -> Result<PathBuf> {
    let base = dirs::config_dir().ok_or_else(|| eyre!("cannot find config directory"))?;
    Ok(base.join("deploykit"))
}

fn profiles_path() -> Result<PathBuf> {
    Ok(config_dir()?.join("profiles.toml"))
}

fn presets_path() -> Result<PathBuf> {
    Ok(config_dir()?.join("presets.toml"))
}

pub fn ensure_config_dir() -> Result<()> {
    let dir = config_dir()?;
    if !dir.exists() {
        fs::create_dir_all(&dir)?;
    }
    Ok(())
}

pub fn load_profiles() -> Result<Vec<Profile>> {
    let path = profiles_path()?;
    if !path.exists() {
        return Ok(Vec::new());
    }
    let data = fs::read_to_string(&path)?;
    let profiles: Vec<Profile> = toml::from_str(&data)?;
    Ok(profiles)
}

pub fn save_profiles(profiles: &[Profile]) -> Result<()> {
    ensure_config_dir()?;
    let data = toml::to_string_pretty(profiles)?;
    fs::write(profiles_path()?, data)?;
    Ok(())
}

pub fn load_presets() -> Result<Vec<Preset>> {
    let path = presets_path()?;
    if !path.exists() {
        return Ok(Vec::new());
    }
    let data = fs::read_to_string(&path)?;
    let presets: Vec<Preset> = toml::from_str(&data)?;
    Ok(presets)
}

pub fn save_presets(presets: &[Preset]) -> Result<()> {
    ensure_config_dir()?;
    let data = toml::to_string_pretty(presets)?;
    fs::write(presets_path()?, data)?;
    Ok(())
}
