use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub wallpaper_cmd: String,
    pub save_dir: String,
    pub api_key: Option<String>,
    pub username: Option<String>,
    pub categories: String,
    pub purity: String,
    pub sorting: String,
    pub ratios: String,
    #[serde(default = "default_wallpaper_mode")]
    pub wallpaper_mode: String,
}

fn default_wallpaper_mode() -> String {
    "contain".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            wallpaper_cmd: "hyprctl hyprpaper preload %f; hyprctl hyprpaper wallpaper \"%m,%f\""
                .to_string(),
            save_dir: "~/Pictures/Wallpapers/Wallhaven".to_string(),
            api_key: None,
            username: None,
            categories: "111".to_string(),
            purity: "100".to_string(),
            sorting: "hot".to_string(),
            ratios: "landscape".to_string(),
            wallpaper_mode: default_wallpaper_mode(),
        }
    }
}

pub fn load_config() -> Result<Config> {
    let config_dir = dirs::config_dir()
        .context("Could not find config directory")?
        .join("hypr");
    let config_path = config_dir.join("hyprwallhaven.toml");

    if !config_path.exists() {
        let config = Config::default();
        fs::create_dir_all(&config_dir)?;
        let toml_string = toml::to_string_pretty(&config)?;
        fs::write(&config_path, toml_string)?;
        return Ok(config);
    }

    let content = fs::read_to_string(&config_path)?;
    let config: Config = toml::from_str(&content)?;
    Ok(config)
}

pub fn save_config(config: &Config) -> Result<()> {
    let config_dir = dirs::config_dir()
        .context("Could not find config directory")?
        .join("hypr");
    let config_path = config_dir.join("hyprwallhaven.toml");

    let toml_string = toml::to_string_pretty(config)?;
    fs::write(&config_path, toml_string)?;
    Ok(())
}

pub fn expand_path(path: &str) -> PathBuf {
    if path.starts_with("~") {
        if let Some(home) = dirs::home_dir() {
            return home.join(path.trim_start_matches("~/"));
        }
    }
    PathBuf::from(path)
}
