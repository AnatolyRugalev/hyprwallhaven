use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct State {
    pub wallpapers: HashMap<String, String>, // monitor_name -> file_path
}

pub fn load_state() -> Result<State> {
    let config_dir = dirs::config_dir()
        .context("Could not find config directory")?
        .join("hypr");
    let state_path = config_dir.join("hyprwallhaven_state.toml");

    if !state_path.exists() {
        return Ok(State::default());
    }

    let content = fs::read_to_string(&state_path)?;
    // If parsing fails, return default state instead of crashing
    let state: State = toml::from_str(&content).unwrap_or_default();
    Ok(state)
}

pub fn save_state(state: &State) -> Result<()> {
    let config_dir = dirs::config_dir()
        .context("Could not find config directory")?
        .join("hypr");
    fs::create_dir_all(&config_dir)?; 
    let state_path = config_dir.join("hyprwallhaven_state.toml");

    let toml_string = toml::to_string_pretty(state)?;
    fs::write(&state_path, toml_string)?;
    Ok(())
}
