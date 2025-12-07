use anyhow::Result;
use std::io::Write;
use std::process::{Command, Stdio};

pub enum MenuAction {
    Rotate,
    SearchApi,
    SetId,
    Settings,
    Collections,
    OpenCurrent,
    Custom(String),
    None,
}

#[derive(Debug)]
pub enum NavAction {
    Next,
    Prev,
    Random,
    OpenInBrowser,
    SettingsCategory,
    SettingsPurity,
    SettingsSorting,
    Done,
    Cancel,
    None,
}

pub enum SettingsAction {
    Categories,
    Purity,
    Sorting,
    WallpaperMode,
    SetApiKey,
    Back,
    None,
}

pub fn show_fuzzel_menu(show_current: bool) -> Result<MenuAction> {
    let mut options = String::from("🎲 Rotate\n🔍 Search\n📚 Collections\n🆔 Set ID/URL\n⚙️ Settings\n");
    let mut lines = 4;
    if show_current {
        options.push_str("👁️ Show Current Wallpaper\n");
        lines = 5;
    }

    let mut child = Command::new("fuzzel")
        .arg("--dmenu")
        .arg(format!("--lines={}", lines))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(options.as_bytes())?;
    }

    let output = child.wait_with_output()?;

    if !output.status.success() {
        return Ok(MenuAction::None);
    }

    let selection = String::from_utf8_lossy(&output.stdout).trim().to_string();

    match selection.as_str() {
        s if s.contains("Rotate") => Ok(MenuAction::Rotate),
        s if s.contains("Search") => Ok(MenuAction::SearchApi),
        s if s.contains("Collections") => Ok(MenuAction::Collections),
        s if s.contains("Set ID") => Ok(MenuAction::SetId),
        s if s.contains("Settings") => Ok(MenuAction::Settings),
        s if s.contains("Show Current Wallpaper") => Ok(MenuAction::OpenCurrent),
        "" => Ok(MenuAction::None),
        _ => Ok(MenuAction::Custom(selection)),
    }
}

pub fn show_settings_menu(
    categories: &str,
    purity: &str,
    sorting: &str,
    wallpaper_mode: &str,
) -> Result<SettingsAction> {
    let mut cat_list = Vec::new();
    if categories.chars().nth(0).unwrap_or('0') == '1' {
        cat_list.push("General");
    }
    if categories.chars().nth(1).unwrap_or('0') == '1' {
        cat_list.push("Anime");
    }
    if categories.chars().nth(2).unwrap_or('0') == '1' {
        cat_list.push("People");
    }
    let cat_str = if cat_list.is_empty() {
        "None".to_string()
    } else {
        cat_list.join(", ")
    };

    let mut purity_list = Vec::new();
    if purity.chars().nth(0).unwrap_or('0') == '1' {
        purity_list.push("SFW");
    }
    if purity.chars().nth(1).unwrap_or('0') == '1' {
        purity_list.push("Sketchy");
    }
    if purity.chars().nth(2).unwrap_or('0') == '1' {
        purity_list.push("NSFW");
    }
    let purity_str = if purity_list.is_empty() {
        "None".to_string()
    } else {
        purity_list.join(", ")
    };

    let options = format!(
        "📂 Categories [{}]\n🔞 Purity [{}]\n📶 Sorting [{}]\n🖼️ Wallpaper Mode [{}]\n🔑 Set API Key\n🔙 Back\n",
        cat_str, purity_str, sorting, wallpaper_mode
    );

    let mut child = Command::new("fuzzel")
        .arg("--dmenu")
        .arg("-p")
        .arg("Settings: ")
        .arg("--lines=6")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(options.as_bytes())?;
    }

    let output = child.wait_with_output()?;

    if !output.status.success() {
        return Ok(SettingsAction::None);
    }

    let selection = String::from_utf8_lossy(&output.stdout).trim().to_string();

    match selection.as_str() {
        s if s.contains("Categories") => Ok(SettingsAction::Categories),
        s if s.contains("Purity") => Ok(SettingsAction::Purity),
        s if s.contains("Sorting") => Ok(SettingsAction::Sorting),
        s if s.contains("Wallpaper Mode") => Ok(SettingsAction::WallpaperMode),
        s if s.contains("Set API Key") => Ok(SettingsAction::SetApiKey),
        s if s.contains("Back") => Ok(SettingsAction::Back),
        _ => Ok(SettingsAction::None),
    }
}

pub fn show_wallpaper_mode_menu(_current: &str) -> Result<Option<String>> {
    let options = "contain\ncover\nfill\ntile\n🔙 Back\n";

    let mut child = Command::new("fuzzel")
        .arg("--dmenu")
        .arg("-p")
        .arg("Wallpaper Mode: ")
        .arg("--lines=5")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(options.as_bytes())?;
    }

    let output = child.wait_with_output()?;

    if !output.status.success() {
        return Ok(None);
    }

    let selection = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if selection.contains("Back") || selection.is_empty() {
        return Ok(None);
    }

    Ok(Some(selection))
}

// ... (keeping other functions)

pub fn get_password_input(prompt: &str) -> Result<String> {
    let child = Command::new("fuzzel")
        .arg("--dmenu")
        .arg("-p")
        .arg(prompt)
        .arg("--lines=0")
        .arg("--password")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    // Write nothing to stdin

    let output = child.wait_with_output()?;

    let input = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(input)
}

pub fn show_categories_menu(current: &str) -> Result<Option<String>> {
    let gen = if current.chars().nth(0).unwrap_or('0') == '1' {
        "ON"
    } else {
        "OFF"
    };
    let anime = if current.chars().nth(1).unwrap_or('0') == '1' {
        "ON"
    } else {
        "OFF"
    };
    let people = if current.chars().nth(2).unwrap_or('0') == '1' {
        "ON"
    } else {
        "OFF"
    };

    let options = format!(
        "Toggle ⬜ General [{}]\nToggle 🎭 Anime [{}]\nToggle 👤 People [{}]\n🔙 Back\n",
        gen, anime, people
    );

    let mut child = Command::new("fuzzel")
        .arg("--dmenu")
        .arg("-p")
        .arg("Categories: ")
        .arg("--lines=4")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(options.as_bytes())?;
    }

    let output = child.wait_with_output()?;

    if !output.status.success() {
        return Ok(None);
    }

    let selection = String::from_utf8_lossy(&output.stdout).trim().to_string();

    let mut chars: Vec<char> = current.chars().collect();
    while chars.len() < 3 {
        chars.push('0');
    }

    if selection.contains("⬜") {
        chars[0] = if chars[0] == '1' { '0' } else { '1' };
    } else if selection.contains("🎭") {
        chars[1] = if chars[1] == '1' { '0' } else { '1' };
    } else if selection.contains("👤") {
        chars[2] = if chars[2] == '1' { '0' } else { '1' };
    } else {
        return Ok(None); // Back or invalid
    }

    Ok(Some(chars.into_iter().collect()))
}

pub fn show_purity_menu(current: &str) -> Result<Option<String>> {
    let sfw = if current.chars().nth(0).unwrap_or('0') == '1' {
        "ON"
    } else {
        "OFF"
    };
    let sketchy = if current.chars().nth(1).unwrap_or('0') == '1' {
        "ON"
    } else {
        "OFF"
    };
    let nsfw = if current.chars().nth(2).unwrap_or('0') == '1' {
        "ON"
    } else {
        "OFF"
    };

    let options = format!(
        "Toggle 🟢 SFW [{}]\nToggle 🟡 Sketchy [{}]\nToggle 🔴 NSFW [{}]\n🔙 Back\n",
        sfw, sketchy, nsfw
    );

    let mut child = Command::new("fuzzel")
        .arg("--dmenu")
        .arg("-p")
        .arg("Purity: ")
        .arg("--lines=4")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(options.as_bytes())?;
    }

    let output = child.wait_with_output()?;

    if !output.status.success() {
        return Ok(None);
    }

    let selection = String::from_utf8_lossy(&output.stdout).trim().to_string();

    let mut chars: Vec<char> = current.chars().collect();
    while chars.len() < 3 {
        chars.push('0');
    }

    if selection.contains("🟢") {
        chars[0] = if chars[0] == '1' { '0' } else { '1' };
    } else if selection.contains("🟡") {
        chars[1] = if chars[1] == '1' { '0' } else { '1' };
    } else if selection.contains("🔴") {
        chars[2] = if chars[2] == '1' { '0' } else { '1' };
    } else {
        return Ok(None);
    }

    Ok(Some(chars.into_iter().collect()))
}

pub fn show_sorting_menu(_current: &str) -> Result<Option<String>> {
    let options = "🎯 relevance\n🎲 random\n📅 date_added\n👁️ views\n❤️ favorites\n🏆 toplist\n🔥 hot\n🔙 Back\n";

    let mut child = Command::new("fuzzel")
        .arg("--dmenu")
        .arg("-p")
        .arg("Sorting: ")
        .arg("--lines=8")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(options.as_bytes())?;
    }

    let output = child.wait_with_output()?;

    if !output.status.success() {
        return Ok(None);
    }

    let selection = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if selection.contains("Back") || selection.is_empty() {
        return Ok(None);
    }

    // Strip emoji
    let clean = selection
        .split_whitespace()
        .last()
        .unwrap_or(&selection)
        .to_string();

    Ok(Some(clean))
}

pub fn show_search_nav_menu(
    current: usize,
    total: usize,
    _has_api_key: bool,
    categories: &str,
    purity: &str,
    sorting: &str,
) -> Result<NavAction> {
    let mut cat_list = Vec::new();
    if categories.chars().nth(0).unwrap_or('0') == '1' {
        cat_list.push("General");
    }
    if categories.chars().nth(1).unwrap_or('0') == '1' {
        cat_list.push("Anime");
    }
    if categories.chars().nth(2).unwrap_or('0') == '1' {
        cat_list.push("People");
    }
    let cat_str = if cat_list.is_empty() {
        "None".to_string()
    } else {
        cat_list.join(", ")
    };

    let mut purity_list = Vec::new();
    if purity.chars().nth(0).unwrap_or('0') == '1' {
        purity_list.push("SFW");
    }
    if purity.chars().nth(1).unwrap_or('0') == '1' {
        purity_list.push("Sketchy");
    }
    if purity.chars().nth(2).unwrap_or('0') == '1' {
        purity_list.push("NSFW");
    }
    let purity_str = if purity_list.is_empty() {
        "None".to_string()
    } else {
        purity_list.join(", ")
    };

    let mut options = String::from("➡️ Next\n⬅️ Prev\n✅ Done\n🎲 Random\n🌐 Open in Browser\n");
    options.push_str(&format!("📂 Category [{}]\n", cat_str));
    options.push_str(&format!("🔞 Purity [{}]\n", purity_str));
    options.push_str(&format!("📶 Sorting [{}]\n", sorting));

    let prompt = format!("Result {}/{}: ", current + 1, total);

    let mut child = Command::new("fuzzel")
        .arg("--dmenu")
        .arg("-p")
        .arg(prompt)
        .arg("--lines=10")
        .arg("--anchor=bottom")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(options.as_bytes())?;
    }

    let output = child.wait_with_output()?;

    if !output.status.success() {
        return Ok(NavAction::Cancel); // Escaping fuzzel usually implies cancelling
    }

    let selection = String::from_utf8_lossy(&output.stdout).trim().to_string();

    match selection.as_str() {
        s if s.contains("Next") => Ok(NavAction::Next),
        s if s.contains("Prev") => Ok(NavAction::Prev),
        s if s.contains("Random") => Ok(NavAction::Random),
        s if s.contains("Open in Browser") => Ok(NavAction::OpenInBrowser),
        s if s.contains("Open in Browser") => Ok(NavAction::OpenInBrowser),
        s if s.contains("Category") => Ok(NavAction::SettingsCategory),
        s if s.contains("Purity") => Ok(NavAction::SettingsPurity),
        s if s.contains("Sorting") => Ok(NavAction::SettingsSorting),
        s if s.contains("Done") => Ok(NavAction::Done),
        _ => Ok(NavAction::None),
    }
}

pub fn show_selection_menu(prompt: &str, items: &[String]) -> Result<Option<String>> {
    let mut options = String::new();
    for item in items {
        options.push_str(item);
        options.push('\n');
    }
    options.push_str("❌ Cancel\n");

    let mut child = Command::new("fuzzel")
        .arg("--dmenu")
        .arg("-p")
        .arg(prompt)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(options.as_bytes())?;
    }

    let output = child.wait_with_output()?;

    if !output.status.success() {
        return Ok(None);
    }

    let selection = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if selection.contains("Cancel") || selection.is_empty() {
        return Ok(None);
    }

    Ok(Some(selection))
}

pub fn show_preview_menu() -> Result<NavAction> {
    let options = "✅ Done\n❌ Cancel\n🌐 Open in Browser\n";

    let mut child = Command::new("fuzzel")
        .arg("--dmenu")
        .arg("-p")
        .arg("Action: ")
        .arg("--lines=3")
        .arg("--anchor=bottom")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(options.as_bytes())?;
    }

    let output = child.wait_with_output()?;

    if !output.status.success() {
        return Ok(NavAction::Cancel);
    }

    let selection = String::from_utf8_lossy(&output.stdout).trim().to_string();

    match selection.as_str() {
        s if s.contains("Done") => Ok(NavAction::Done),
        s if s.contains("Cancel") => Ok(NavAction::Cancel),
        s if s.contains("Open in Browser") => Ok(NavAction::OpenInBrowser),
        _ => Ok(NavAction::None),
    }
}

pub fn get_user_input(prompt: &str) -> Result<String> {
    // Try using --prompt-only first
    let child = Command::new("fuzzel")
        .arg("--dmenu")
        .arg("-p")
        .arg(prompt)
        .arg("--prompt-only")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let output = child.wait_with_output()?;

    if output.status.success() {
        let input = String::from_utf8_lossy(&output.stdout).trim().to_string();
        return Ok(input);
    }

    // Fallback if --prompt-only failed (likely unsupported version)
    eprintln!("Warning: 'fuzzel --prompt-only' failed, falling back to standard dmenu mode. Update fuzzel for better experience.");

    let child = Command::new("fuzzel")
        .arg("--dmenu")
        .arg("-p")
        .arg(prompt)
        .arg("--lines=0")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    // We don't write anything to stdin, effectively an empty list
    let output = child.wait_with_output()?;

    let input = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(input)
}
