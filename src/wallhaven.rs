use crate::config::Config;
use anyhow::Result;
use reqwest::blocking::Client;
use serde::Deserialize;
use std::fs;
use std::io::copy;
use std::path::Path;

#[derive(Deserialize, Debug, Clone)]
pub struct Wallpaper {
    pub id: String,
    pub short_url: String,
    pub path: String, // API returns 'path' as the full image url usually
}

#[derive(Deserialize, Debug, Clone)]
pub struct Collection {
    pub id: i64,
    pub label: String,
}

#[derive(Deserialize, Debug)]
struct CollectionsResponse {
    data: Vec<Collection>,
}

#[derive(Deserialize, Debug)]
struct SettingsResponse {
    data: UserSettings,
}

#[derive(Deserialize, Debug)]
struct UserSettings {
    username: String,
}

#[derive(Deserialize, Debug)]
struct SearchResponse {
    data: Vec<Wallpaper>,
}

#[derive(Deserialize, Debug)]
struct ImageResponse {
    data: Wallpaper,
}

use reqwest::StatusCode;

// ...

pub fn search_wallpapers(
    config: &Config,
    query: Option<&str>,
    page: u32,
    ratio_override: Option<&str>,
) -> Result<Vec<Wallpaper>> {
    let client = Client::new();
    let ratios = ratio_override.unwrap_or(&config.ratios);
    let mut url = format!(
        "https://wallhaven.cc/api/v1/search?categories={}&purity={}&sorting={}&ratios={}&page={}",
        config.categories, config.purity, config.sorting, ratios, page
    );

    if let Some(q) = query {
        url.push_str(&format!("&q={}", q));
    }

    if let Some(key) = &config.api_key {
        url.push_str(&format!("&apikey={}", key));
    }

    let resp = client.get(&url).send()?;

    if resp.status() == StatusCode::UNAUTHORIZED {
        anyhow::bail!("401 Unauthorized: API Key required/invalid. Please set 'api_key' in ~/.config/hypr/wallhaven.toml for NSFW/Restricted content.");
    }

    let resp = resp.error_for_status()?.json::<SearchResponse>()?;
    Ok(resp.data)
}

pub fn get_wallpaper_info(id: &str, config: &Config) -> Result<Wallpaper> {
    let client = Client::new();
    let mut url = format!("https://wallhaven.cc/api/v1/w/{}", id);
    if let Some(key) = &config.api_key {
        url.push_str(&format!("?apikey={}", key));
    }

    let resp = client.get(&url).send()?;

    if resp.status() == StatusCode::UNAUTHORIZED {
        anyhow::bail!("401 Unauthorized: API Key required/invalid. Please set 'api_key' in ~/.config/hypr/wallhaven.toml to access restricted wallpaper (ID: {}).", id);
    }

    let resp = resp.error_for_status()?.json::<ImageResponse>()?;
    Ok(resp.data)
}

pub fn download_wallpaper(url: &str, path: &Path) -> Result<()> {
    if path.exists() {
        return Ok(());
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut response = reqwest::blocking::get(url)?;
    let mut file = fs::File::create(path)?;
    copy(&mut response, &mut file)?;
    Ok(())
}

pub fn get_my_collections(config: &Config) -> Result<Vec<Collection>> {
    let client = Client::new();
    let mut url = "https://wallhaven.cc/api/v1/collections".to_string();

    if let Some(key) = &config.api_key {
        url.push_str(&format!("?apikey={}", key));
    } else {
        anyhow::bail!("API Key required to fetch collections");
    }

    let resp = client.get(&url).send()?;
    let text = resp.error_for_status()?.text()?;
    eprintln!("DEBUG: Collections response: {}", text);
    let resp: CollectionsResponse = serde_json::from_str(&text)?;
    Ok(resp.data)
}


pub fn get_collection_wallpapers(
    config: &Config,
    username: &str,
    collection_id: i64,
    page: u32,
) -> Result<Vec<Wallpaper>> {
    let client = Client::new();
    let mut url = format!(
        "https://wallhaven.cc/api/v1/collections/{}/{}?page={}",
        username, collection_id, page
    );

    if let Some(key) = &config.api_key {
        url.push_str(&format!("&apikey={}", key));
    } else {
        // Collections might be public, but let's stick to consistent API usage if key is present
        // If it's private collection, we need key.
        // If we want to view user's own private collections, we definitely need it.
    }

    let resp = client.get(&url).send()?;
    
    if resp.status() == StatusCode::UNAUTHORIZED {
        anyhow::bail!("401 Unauthorized: API Key required/invalid.");
    }

    let resp = resp.error_for_status()?.json::<SearchResponse>()?;
    Ok(resp.data)
}

pub fn get_username(config: &Config) -> Result<String> {
    let client = Client::new();
    let mut url = "https://wallhaven.cc/api/v1/settings".to_string();

    if let Some(key) = &config.api_key {
        url.push_str(&format!("?apikey={}", key));
    } else {
        anyhow::bail!("API Key required to fetch username");
    }

    let resp = client.get(&url).send()?;
    let text = resp.error_for_status()?.text()?;
    eprintln!("DEBUG: Settings response: {}", text);
    let resp: SettingsResponse = serde_json::from_str(&text)?;
    Ok(resp.data.username)
}

