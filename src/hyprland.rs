use anyhow::{Context, Result};
use serde::Deserialize;
use std::process::Command;

#[derive(Deserialize, Debug)]
pub struct ActiveWorkspace {
    pub id: i64,
}

#[derive(Deserialize, Debug)]
pub struct Monitor {
    pub name: String,
    pub width: i32,
    pub height: i32,
    pub focused: bool,
    #[serde(default)]
    pub transform: i32,
    #[serde(rename = "activeWorkspace")]
    pub active_workspace: ActiveWorkspace,
}

impl Monitor {
    pub fn get_visual_dimensions(&self) -> (i32, i32) {
        // Hyprland transform values:
        // 0: Normal
        // 1: 90 degrees
        // 2: 180 degrees
        // 3: 270 degrees
        // 4: Flipped
        // 5: Flipped + 90
        // 6: Flipped + 180
        // 7: Flipped + 270
        match self.transform {
            1 | 3 | 5 | 7 => (self.height, self.width),
            _ => (self.width, self.height),
        }
    }
}

#[derive(Deserialize, Debug)]
struct Workspace {
    id: i64,
}

pub fn get_active_monitor() -> Result<Monitor> {
    let output = Command::new("hyprctl")
        .arg("monitors")
        .arg("-j")
        .output()
        .context("Failed to execute hyprctl monitors -j")?;

    if !output.status.success() {
        anyhow::bail!("hyprctl monitors -j failed");
    }

    let monitors: Vec<Monitor> =
        serde_json::from_slice(&output.stdout).context("Failed to parse hyprctl output")?;

    monitors
        .into_iter()
        .find(|m| m.focused)
        .ok_or_else(|| anyhow::anyhow!("No focused monitor found"))
}

pub fn get_occupied_workspaces() -> Result<Vec<i64>> {
    let output = Command::new("hyprctl")
        .arg("workspaces")
        .arg("-j")
        .output()
        .context("Failed to execute hyprctl workspaces -j")?;

    if !output.status.success() {
        anyhow::bail!("hyprctl workspaces -j failed");
    }

    let workspaces: Vec<Workspace> =
        serde_json::from_slice(&output.stdout).context("Failed to parse hyprctl output")?;

    Ok(workspaces.into_iter().map(|w| w.id).collect())
}

pub fn dispatch_workspace(id: i64) -> Result<()> {
    let output = Command::new("hyprctl")
        .arg("dispatch")
        .arg("workspace")
        .arg(id.to_string())
        .output()
        .context("Failed to switch workspace")?;

    if !output.status.success() {
        anyhow::bail!("Failed to switch workspace");
    }
    Ok(())
}

pub fn get_current_wallpaper(monitor_name: &str) -> Result<String> {
    let output = Command::new("hyprctl")
        .arg("hyprpaper")
        .arg("listactive")
        .output()
        .context("Failed to execute hyprctl hyprpaper listactive")?;

    if !output.status.success() {
        anyhow::bail!("hyprctl hyprpaper listactive failed");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Output format: "MONITOR = IMAGE" (one per line)
    // e.g. "DP-1 = /path/to/image.png"

    for line in stdout.lines() {
        if let Some((mon, path)) = line.split_once('=') {
            if mon.trim() == monitor_name {
                return Ok(path.trim().to_string());
            }
        }
    }

    anyhow::bail!("Current wallpaper for monitor {} not found", monitor_name);
}
